use jlrs::prelude::*;

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, ValidField, ConstructType, CCallArg)]
#[jlrs(julia_type = "PromptVeilCore.CompressedResult")]
pub struct CompressedResult<'scope, 'data> {
    pub data: ::std::option::Option<::jlrs::data::managed::array::ArrayRef<'scope, 'data>>,
    pub original_size: i64,
    pub compressed_size: i64,
}

#[repr(C)]
#[derive(Clone, Debug, Unbox, ValidLayout, Typecheck, IntoJulia, ValidField, IsBits, ConstructType, CCallArg, CCallReturn)]
#[jlrs(julia_type = "PromptVeilCore.CompressionConfig")]
pub struct CompressionConfig {
    pub use_gpu: ::jlrs::data::layout::bool::Bool,
    pub use_simd: ::jlrs::data::layout::bool::Bool,
    pub use_patterns: ::jlrs::data::layout::bool::Bool,
}