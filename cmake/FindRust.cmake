# FindRust.cmake
# Finds Rust compiler and cargo
#
# This will define:
# Rust_FOUND - True if Rust was found
# Rust_COMPILER - Path to rustc
# Rust_CARGO - Path to cargo
# Rust_VERSION - Rust version

# Find rustc
find_program(Rust_COMPILER
    NAMES rustc rustc.exe
    DOC "Rust compiler"
)

# Find cargo
find_program(Rust_CARGO
    NAMES cargo cargo.exe
    DOC "Cargo package manager"
)

if(Rust_COMPILER)
    # Get Rust version
    execute_process(
        COMMAND ${Rust_COMPILER} --version
        OUTPUT_VARIABLE Rust_VERSION_STRING
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )
    
    string(REGEX MATCH "[0-9]+\\.[0-9]+\\.[0-9]+" Rust_VERSION "${Rust_VERSION_STRING}")
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(Rust
    REQUIRED_VARS
        Rust_COMPILER
        Rust_CARGO
    VERSION_VAR
        Rust_VERSION
)

# Helper function to build Rust crates
function(add_rust_target target_name cargo_toml working_dir build_type)
    # Determine build type flags
    if(${build_type} STREQUAL "debug")
        set(CARGO_BUILD_FLAGS "")
    else()
        set(CARGO_BUILD_FLAGS "--release")
    endif()

    # Add PyO3 features if Python is found
    if(Python3_FOUND)
        set(CARGO_FEATURES "pyo3")
    endif()

    # Add GPU features if enabled
    if(USE_GPU)
        set(CARGO_FEATURES "${CARGO_FEATURES},gpu")
    endif()

    # Build command with features
    if(DEFINED CARGO_FEATURES)
        set(FEATURE_FLAGS "--features=${CARGO_FEATURES}")
    else()
        set(FEATURE_FLAGS "")
    endif()

    add_custom_target(${target_name}
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${CMAKE_BINARY_DIR}/cargo_target
            ${Rust_CARGO} build ${CARGO_BUILD_FLAGS} ${FEATURE_FLAGS}
        WORKING_DIRECTORY ${working_dir}
        DEPENDS ${cargo_toml}
        COMMENT "Building Rust target ${target_name} (${build_type})"
    )

    # Add clean target
    add_custom_target(${target_name}_clean
        COMMAND ${Rust_CARGO} clean
        WORKING_DIRECTORY ${working_dir}
    )
endfunction() 