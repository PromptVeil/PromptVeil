#ifndef JULIA_INIT_H
#define JULIA_INIT_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// Token optimization functions
uint32_t* julia_optimize_tokens(const uint32_t* tokens, int64_t len);
uint32_t* julia_compress_batch(const uint32_t* tokens, int64_t rows, int64_t cols);

#ifdef __cplusplus
}
#endif

#endif // JULIA_INIT_H 