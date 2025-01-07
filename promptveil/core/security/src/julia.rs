use std::os::raw::{c_void, c_int};

#[link(name = "PromptVeilCore")]
extern "C" {
    pub fn julia_init();
    pub fn julia_cleanup();
    
    pub fn julia_optimize_tokens_config(
        ptr: *const u32,
        len: i64,
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> *mut u32;

    pub fn julia_compress_batch_config(
        ptr: *const u32,
        rows: i64,
        cols: i64,
        use_gpu: bool,
        use_simd: bool,
        use_patterns: bool
    ) -> *mut u32;

    pub fn julia_decompress_batch(
        ptr: *const u32,
        rows: i64,
        cols: i64
    ) -> *mut u32;
} 