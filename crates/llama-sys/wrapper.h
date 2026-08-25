#include "llama.h"
#include "ggml-backend.h"

static inline size_t llama_rs_backend_dev_count(void) {
    return ggml_backend_dev_count();
}

static inline ggml_backend_dev_t llama_rs_backend_dev_get(size_t index) {
    return ggml_backend_dev_get(index);
}

static inline const char * llama_rs_backend_dev_name(ggml_backend_dev_t device) {
    return ggml_backend_dev_name(device);
}

static inline const char * llama_rs_backend_dev_description(ggml_backend_dev_t device) {
    return ggml_backend_dev_description(device);
}

static inline const char * llama_rs_backend_reg_name_for_device(ggml_backend_dev_t device) {
    ggml_backend_reg_t reg = ggml_backend_dev_backend_reg(device);
    return reg == NULL ? NULL : ggml_backend_reg_name(reg);
}

static inline bool llama_rs_backend_dev_is_gpu(ggml_backend_dev_t device) {
    enum ggml_backend_dev_type type = ggml_backend_dev_type(device);
    return type == GGML_BACKEND_DEVICE_TYPE_GPU || type == GGML_BACKEND_DEVICE_TYPE_IGPU;
}
