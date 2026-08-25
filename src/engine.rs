use std::ffi::CString;
use std::io::Write;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::ptr;

use anyhow::{bail, Context, Result};

use crate::runtime_backend::{select_devices_for_model, RuntimeBackend, RuntimeDevice};

/// Offload all model layers that llama.cpp can place on the selected GPU backend.
pub const DEFAULT_GPU_LAYERS: u32 = 999;

/// How to load a GGUF model.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    pub model_path: PathBuf,
    pub mmproj_path: Option<PathBuf>,
    pub ctx_size: u32,
    pub threads: i32,
    pub gpu_layers: u32,
}

impl EngineConfig {
    pub fn new(model_path: impl Into<PathBuf>) -> Self {
        Self {
            model_path: model_path.into(),
            mmproj_path: None,
            ctx_size: 2048,
            threads: 0,
            gpu_layers: DEFAULT_GPU_LAYERS,
        }
    }

    pub fn with_mmproj(mut self, mmproj_path: impl Into<PathBuf>) -> Self {
        self.mmproj_path = Some(mmproj_path.into());
        self
    }

    pub fn with_ctx_size(mut self, ctx_size: u32) -> Self {
        self.ctx_size = ctx_size;
        self
    }

    pub fn with_threads(mut self, threads: i32) -> Self {
        self.threads = threads;
        self
    }

    pub fn with_gpu_layers(mut self, gpu_layers: u32) -> Self {
        self.gpu_layers = gpu_layers;
        self
    }
}

/// Generation settings for one prompt.
#[derive(Debug, Clone)]
pub struct GenerateRequest {
    pub prompt: String,
    pub max_tokens: i32,
    pub temperature: f32,
    pub seed: u32,
    pub image_path: Option<PathBuf>,
    pub chat: bool,
}

impl GenerateRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            max_tokens: 128,
            temperature: 0.8,
            seed: 42,
            image_path: None,
            chat: false,
        }
    }

    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    pub fn with_image(mut self, image_path: impl Into<PathBuf>) -> Self {
        self.image_path = Some(image_path.into());
        self
    }

    pub fn with_chat(mut self, chat: bool) -> Self {
        self.chat = chat;
        self
    }
}

/// Loaded GGUF model that can run multiple generations.
pub struct LlamaEngine {
    model: *mut llama_sys::llama_model,
    ctx_size: u32,
    threads: i32,
    model_path: PathBuf,
    mmproj_path: Option<PathBuf>,
    active_backend: RuntimeBackend,
    runtime_devices: Vec<RuntimeDevice>,
}

unsafe impl Send for LlamaEngine {}
unsafe impl Sync for LlamaEngine {}

impl LlamaEngine {
    pub fn load(config: EngineConfig) -> Result<Self> {
        if let Some(mmproj) = &config.mmproj_path {
            if !mmproj.exists() {
                bail!("mmproj does not exist: {}", mmproj.display());
            }
        }

        let mut selected = select_devices_for_model(config.gpu_layers);
        let path = CString::new(config.model_path.to_string_lossy().as_bytes())
            .context("model path contains interior nul")?;

        let model = unsafe {
            let mut params = llama_sys::llama_model_default_params();
            params.n_gpu_layers = config.gpu_layers as i32;
            if !selected.devices.is_empty() {
                params.devices = selected.devices.as_mut_ptr();
            }
            llama_sys::llama_model_load_from_file(path.as_ptr(), params)
        };
        if model.is_null() {
            bail!("failed to load model: {}", config.model_path.display());
        }

        Ok(Self {
            model,
            ctx_size: config.ctx_size,
            threads: config.threads,
            model_path: config.model_path,
            mmproj_path: config.mmproj_path,
            active_backend: selected.backend,
            runtime_devices: selected.snapshot,
        })
    }

    pub fn model_path(&self) -> &Path {
        &self.model_path
    }

    pub fn mmproj_path(&self) -> Option<&Path> {
        self.mmproj_path.as_deref()
    }

    pub fn is_vision_model(&self) -> bool {
        self.mmproj_path.is_some()
    }

    pub fn active_backend(&self) -> &RuntimeBackend {
        &self.active_backend
    }

    pub fn runtime_devices(&self) -> &[RuntimeDevice] {
        &self.runtime_devices
    }

    pub fn generate(&self, request: &GenerateRequest) -> Result<String> {
        self.generate_with_callback(request, |_| {})
    }

    pub fn generate_to_writer(
        &self,
        request: &GenerateRequest,
        writer: &mut impl Write,
    ) -> Result<String> {
        self.generate_with_callback(request, |piece| {
            let _ = write!(writer, "{piece}");
            let _ = writer.flush();
        })
    }

    pub fn generate_with_callback<F>(
        &self,
        request: &GenerateRequest,
        mut on_piece: F,
    ) -> Result<String>
    where
        F: FnMut(&str),
    {
        if request.image_path.is_some() && self.mmproj_path.is_none() {
            bail!("--image requires a vision model mmproj file");
        }
        if let Some(image) = &request.image_path {
            if !image.exists() {
                bail!("image does not exist: {}", image.display());
            }
        }

        let prompt = model_prompt(request, self.mmproj_path.as_deref());
        let ctx = unsafe { self.new_context()? };
        let vocab = unsafe { llama_sys::llama_model_get_vocab(self.model) };
        if vocab.is_null() {
            unsafe { llama_sys::llama_free(ctx) };
            bail!("model has no vocab");
        }

        let tokens = match unsafe { tokenize(vocab, &prompt, true) } {
            Ok(tokens) => tokens,
            Err(err) => {
                unsafe { llama_sys::llama_free(ctx) };
                return Err(err);
            }
        };
        if tokens.is_empty() {
            unsafe { llama_sys::llama_free(ctx) };
            bail!("prompt produced no tokens");
        }

        let n_ctx = unsafe { llama_sys::llama_n_ctx(ctx) } as i32;
        if tokens.len() as i32 + request.max_tokens > n_ctx {
            unsafe { llama_sys::llama_free(ctx) };
            bail!(
                "prompt + max-tokens ({}) exceeds context size ({n_ctx})",
                tokens.len() as i32 + request.max_tokens
            );
        }

        let result = unsafe { generate_loop(ctx, vocab, &tokens, request, &mut on_piece) };
        unsafe { llama_sys::llama_free(ctx) };
        result
    }

    unsafe fn new_context(&self) -> Result<*mut llama_sys::llama_context> {
        let mut params = llama_sys::llama_context_default_params();
        params.n_ctx = self.ctx_size;
        params.n_batch = self.ctx_size.max(512);
        if self.threads > 0 {
            params.n_threads = self.threads;
            params.n_threads_batch = self.threads;
        }
        let ctx = llama_sys::llama_init_from_model(self.model, params);
        if ctx.is_null() {
            bail!("failed to create llama.cpp context");
        }
        Ok(ctx)
    }
}

impl Drop for LlamaEngine {
    fn drop(&mut self) {
        if !self.model.is_null() {
            unsafe { llama_sys::llama_model_free(self.model) };
            self.model = ptr::null_mut();
        }
    }
}

fn model_prompt(request: &GenerateRequest, mmproj: Option<&Path>) -> String {
    let user = match (&request.image_path, mmproj) {
        (Some(image), Some(mmproj)) => format!(
            "Image file: {}\nVision projector: {}\n{}",
            image.display(),
            mmproj.display(),
            request.prompt
        ),
        _ => request.prompt.clone(),
    };

    if request.chat {
        format!(
            "<|im_start|>system\nYou are a helpful assistant. Answer the question in one or two short sentences.<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
        )
    } else {
        user
    }
}

unsafe fn tokenize(
    vocab: *const llama_sys::llama_vocab,
    text: &str,
    add_special: bool,
) -> Result<Vec<llama_sys::llama_token>> {
    let bytes = text.as_bytes();
    let mut n_tokens = llama_sys::llama_tokenize(
        vocab,
        bytes.as_ptr() as *const c_char,
        bytes.len() as i32,
        ptr::null_mut(),
        0,
        add_special,
        true,
    );
    if n_tokens == i32::MIN {
        bail!("tokenize overflow");
    }
    if n_tokens < 0 {
        n_tokens = -n_tokens;
    }
    let mut tokens = vec![0; n_tokens as usize];
    let written = llama_sys::llama_tokenize(
        vocab,
        bytes.as_ptr() as *const c_char,
        bytes.len() as i32,
        tokens.as_mut_ptr(),
        n_tokens,
        add_special,
        true,
    );
    if written < 0 {
        bail!("failed to tokenize prompt");
    }
    tokens.truncate(written as usize);
    Ok(tokens)
}

unsafe fn token_piece(
    vocab: *const llama_sys::llama_vocab,
    token: llama_sys::llama_token,
) -> Result<String> {
    let mut buf = vec![0u8; 256];
    let n = llama_sys::llama_token_to_piece(
        vocab,
        token,
        buf.as_mut_ptr() as *mut c_char,
        buf.len() as i32,
        0,
        true,
    );
    if n < 0 {
        buf.resize((-n) as usize, 0);
        let n = llama_sys::llama_token_to_piece(
            vocab,
            token,
            buf.as_mut_ptr() as *mut c_char,
            buf.len() as i32,
            0,
            true,
        );
        if n < 0 {
            bail!("failed to decode token");
        }
        buf.truncate(n as usize);
    } else {
        buf.truncate(n as usize);
    }
    Ok(String::from_utf8_lossy(&buf).into_owned())
}

unsafe fn generate_loop<F>(
    ctx: *mut llama_sys::llama_context,
    vocab: *const llama_sys::llama_vocab,
    prompt_tokens: &[llama_sys::llama_token],
    request: &GenerateRequest,
    on_piece: &mut F,
) -> Result<String>
where
    F: FnMut(&str),
{
    let mut tokens = prompt_tokens.to_vec();
    let mut batch = llama_sys::llama_batch_get_one(tokens.as_mut_ptr(), tokens.len() as i32);
    if llama_sys::llama_decode(ctx, batch) != 0 {
        bail!("failed to decode prompt");
    }

    let sparams = llama_sys::llama_sampler_chain_default_params();
    let smpl = llama_sys::llama_sampler_chain_init(sparams);
    if smpl.is_null() {
        bail!("failed to create sampler");
    }
    llama_sys::llama_sampler_chain_add(
        smpl,
        llama_sys::llama_sampler_init_temp(request.temperature),
    );
    llama_sys::llama_sampler_chain_add(smpl, llama_sys::llama_sampler_init_dist(request.seed));

    let mut generated = String::new();
    for _ in 0..request.max_tokens {
        let token = llama_sys::llama_sampler_sample(smpl, ctx, -1);
        llama_sys::llama_sampler_accept(smpl, token);
        if llama_sys::llama_vocab_is_eog(vocab, token) {
            break;
        }
        let piece = match token_piece(vocab, token) {
            Ok(piece) => piece,
            Err(err) => {
                llama_sys::llama_sampler_free(smpl);
                return Err(err);
            }
        };
        if piece.contains("<|im_end|>") {
            generated.push_str(piece.split("<|im_end|>").next().unwrap_or(""));
            break;
        }
        generated.push_str(&piece);
        on_piece(&piece);

        let mut one = [token];
        batch = llama_sys::llama_batch_get_one(one.as_mut_ptr(), 1);
        if llama_sys::llama_decode(ctx, batch) != 0 {
            llama_sys::llama_sampler_free(smpl);
            bail!("failed to decode generated token");
        }
    }

    llama_sys::llama_sampler_free(smpl);
    Ok(generated)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_config_offloads_all_gpu_layers_by_default() {
        let config = EngineConfig::new("model.gguf");
        assert_eq!(config.gpu_layers, DEFAULT_GPU_LAYERS);
        assert_eq!(config.gpu_layers, 999);
    }

    #[test]
    fn engine_config_can_force_cpu_inference() {
        let config = EngineConfig::new("model.gguf").with_gpu_layers(0);
        assert_eq!(config.gpu_layers, 0);
    }
}
