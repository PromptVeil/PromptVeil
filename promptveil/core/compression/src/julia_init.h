#ifndef JULIA_INIT_H
#define JULIA_INIT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Julia runtime initialization
void julia_init(void);
void julia_cleanup(void);

// Token optimization functions
uint32_t* julia_optimize_tokens_config(const uint32_t* tokens, int64_t len, bool use_gpu, bool use_simd, bool use_patterns);
uint32_t* julia_compress_batch_config(const uint32_t* tokens, int64_t rows, int64_t cols, bool use_gpu, bool use_simd, bool use_patterns);
uint32_t* julia_decompress_batch(const uint32_t* tokens, int64_t rows, int64_t cols);

#ifdef __cplusplus
}
#endif

#endif // JULIA_INIT_H 