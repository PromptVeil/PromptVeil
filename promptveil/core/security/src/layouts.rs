use jlrs::prelude::*;
use jlrs::data::layout::bool::Bool;

#[repr(C)]
#[derive(Clone, Debug, CCallArg, CCallReturn)]
pub struct CompressedResult<'scope, 'data> {
    pub data: Option<ArrayRef<'scope, 'data>>,
    pub original_size: i64,
    pub compressed_size: i64,
}

#[repr(C)]
#[derive(Clone, Debug, CCallArg, CCallReturn)]
pub struct CompressionConfig {
    pub use_gpu: Bool,
    pub use_simd: Bool,
    pub use_patterns: Bool,
}