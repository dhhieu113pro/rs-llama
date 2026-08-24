use std::io::{self, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel};
use llama_cpp_2::sampling::LlamaSampler;

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
            gpu_layers: 0,
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
}

impl GenerateRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            max_tokens: 128,
            temperature: 0.8,
            seed: 42,
            image_path: None,
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
}

/// Loaded GGUF model that can run multiple generations.
pub struct LlamaEngine {
    backend: LlamaBackend,
    model: LlamaModel,
    ctx_size: u32,
    threads: i32,
    model_path: PathBuf,
    mmproj_path: Option<PathBuf>,
}

impl LlamaEngine {
    pub fn load(config: EngineConfig) -> Result<Self> {
        if let Some(mmproj) = &config.mmproj_path {
            if !mmproj.exists() {
                bail!("mmproj does not exist: {}", mmproj.display());
            }
        }

        let backend = LlamaBackend::init().context("failed to initialize llama.cpp backend")?;
        let model_params = make_model_params(config.gpu_layers);
        let model = LlamaModel::load_from_file(&backend, &config.model_path, &model_params)
            .with_context(|| format!("failed to load model: {}", config.model_path.display()))?;

        Ok(Self {
            backend,
            model,
            ctx_size: config.ctx_size,
            threads: config.threads,
            model_path: config.model_path,
            mmproj_path: config.mmproj_path,
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

    /// Generate text and return only the completion (not the prompt).
    pub fn generate(&self, request: &GenerateRequest) -> Result<String> {
        self.generate_with_callback(request, |_| {})
    }

    /// Generate text and stream each decoded piece to `writer`.
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

    /// Generate text and call `on_piece` for every decoded fragment.
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

        let n_ctx =
            NonZeroU32::new(self.ctx_size).context("ctx-size must be greater than zero")?;
        let mut ctx_params = LlamaContextParams::default().with_n_ctx(Some(n_ctx));

        if self.threads > 0 {
            ctx_params = ctx_params
                .with_n_threads(self.threads)
                .with_n_threads_batch(self.threads);
        }

        let mut ctx = self
            .model
            .new_context(&self.backend, ctx_params)
            .context("failed to create llama.cpp context")?;

        let prompt = vision_prompt(request, self.mmproj_path.as_deref());
        let prompt_tokens = self
            .model
            .str_to_token(&prompt, AddBos::Always)
            .context("failed to tokenize prompt")?;

        if prompt_tokens.is_empty() {
            bail!("prompt produced no tokens");
        }

        let max_total_tokens = prompt_tokens.len() as i32 + request.max_tokens;
        if max_total_tokens > ctx.n_ctx() as i32 {
            bail!(
                "prompt + max-tokens ({max_total_tokens}) exceeds context size ({})",
                ctx.n_ctx()
            );
        }

        let mut batch = LlamaBatch::new(512, 1);
        let last_prompt_index = prompt_tokens.len() - 1;

        for (i, token) in prompt_tokens.iter().copied().enumerate() {
            batch.add(token, i as i32, &[0], i == last_prompt_index)?;
        }

        ctx.decode(&mut batch).context("failed to decode prompt")?;

        let mut sampler = LlamaSampler::chain_simple([
            LlamaSampler::temp(request.temperature),
            LlamaSampler::dist(request.seed),
        ]);

        let mut decoder = encoding_rs::UTF_8.new_decoder();
        let mut position = prompt_tokens.len() as i32;
        let mut generated = String::new();

        for _ in 0..request.max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            if self.model.is_eog_token(token) {
                break;
            }

            let piece = self
                .model
                .token_to_piece(token, &mut decoder, true, None)?;
            generated.push_str(&piece);
            on_piece(&piece);

            batch.clear();
            batch.add(token, position, &[0], true)?;
            ctx.decode(&mut batch)
                .context("failed to decode generated token")?;
            position += 1;
        }

        let _ = io::stdout();
        Ok(generated)
    }
}

fn vision_prompt(request: &GenerateRequest, mmproj: Option<&Path>) -> String {
    match (&request.image_path, mmproj) {
        (Some(image), Some(mmproj)) => format!(
            "<image>\nImage file: {}\nVision projector: {}\n{}\n",
            image.display(),
            mmproj.display(),
            request.prompt
        ),
        _ => request.prompt.clone(),
    }
}

fn make_model_params(gpu_layers: u32) -> LlamaModelParams {
    let params = LlamaModelParams::default();

    #[cfg(any(feature = "cuda", feature = "vulkan", feature = "metal"))]
    {
        if gpu_layers > 0 {
            return params.with_n_gpu_layers(gpu_layers);
        }
    }

    #[cfg(not(any(feature = "cuda", feature = "vulkan", feature = "metal")))]
    let _ = gpu_layers;

    params
}
