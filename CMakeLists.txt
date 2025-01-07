cmake_minimum_required(VERSION 3.15)
project(PromptVeil VERSION 0.1.0 LANGUAGES CXX)

# Add cmake modules path
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# Options
option(BUILD_TESTS "Build tests" ON)
option(BUILD_DOCS "Build documentation" OFF)
option(BUILD_EXAMPLES "Build examples" OFF)

# Find required packages
find_package(Julia REQUIRED)
find_package(Rust REQUIRED)
find_package(Python3 REQUIRED COMPONENTS Interpreter Development)

# Set variables for Julia
set(JULIA_PROJECT_PATH "${CMAKE_SOURCE_DIR}/promptveil/core/compression")
set(JULIA_BUILD_PATH "${CMAKE_BINARY_DIR}/julia_build")

# Get Julia root directory and set JULIA_DIR globally
get_filename_component(JULIA_ROOT_DIR "${Julia_EXECUTABLE}" DIRECTORY)
get_filename_component(JULIA_ROOT_DIR "${JULIA_ROOT_DIR}" DIRECTORY)

# Set JULIA_DIR for the entire build process
set(ENV{JULIA_DIR} "${JULIA_ROOT_DIR}")

# Set variables for Rust
set(RUST_SECURITY_PATH "${CMAKE_SOURCE_DIR}/promptveil/core/security")
set(RUST_FORMAT_PATH "${CMAKE_SOURCE_DIR}/promptveil/core/format")
set(RUST_BUILD_TYPE $<IF:$<CONFIG:Debug>,debug,release>)

# Set Cargo target directory
set(CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo_target")

# Function to add a Rust target
function(add_rust_target target_name cargo_toml cargo_dir build_type)
    if(WIN32)
        set(CARGO_CMD cargo build --manifest-path ${cargo_toml} $<IF:$<CONFIG:Debug>,,--release>)
    else()
        set(CARGO_CMD cargo build --manifest-path ${cargo_toml} $<IF:$<CONFIG:Debug>,,--release>)
    endif()

    add_custom_target(${target_name} ALL
        COMMAND ${CMAKE_COMMAND} -E env
            CARGO_TARGET_DIR=${CARGO_TARGET_DIR}
            ${CARGO_CMD}
        WORKING_DIRECTORY ${cargo_dir}
        COMMENT "Building Rust target ${target_name} (${build_type})"
    )
endfunction()

# Rust security module with PyO3
add_rust_target(rust_security
    ${RUST_SECURITY_PATH}/Cargo.toml
    ${RUST_SECURITY_PATH}
    ${RUST_BUILD_TYPE}
)

# Set environment variables for Rust security target
set_target_properties(rust_security PROPERTIES
    ENVIRONMENT "JULIA_DIR=${JULIA_ROOT_DIR};PROMPTVEIL_CORE_DIR=${CMAKE_SOURCE_DIR}/promptveil/core/compression;RUST_BACKTRACE=1;CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
)

if(WIN32)
    # Additional Windows-specific environment variables
    set_target_properties(rust_security PROPERTIES
        ENVIRONMENT "JULIA_DIR=${JULIA_ROOT_DIR};PROMPTVEIL_CORE_DIR=${CMAKE_SOURCE_DIR}/promptveil/core/compression;RUST_BACKTRACE=1;CARGO_TARGET_DIR=${CARGO_TARGET_DIR};LIB=${JULIA_BUILD_PATH};RUSTFLAGS=-L${JULIA_BUILD_PATH}"
    )
endif()

# Set environment variables globally for the build process
set(ENV{JULIA_DIR} "${JULIA_ROOT_DIR}")
set(ENV{PROMPTVEIL_CORE_DIR} "${CMAKE_SOURCE_DIR}/promptveil/core/compression")
set(ENV{RUST_BACKTRACE} "1")

# Rust format module with PyO3
add_rust_target(rust_format
    ${RUST_FORMAT_PATH}/Cargo.toml
    ${RUST_FORMAT_PATH}
    ${RUST_BUILD_TYPE}
)

# Set environment variables for format module
set_target_properties(rust_format PROPERTIES
    ENVIRONMENT "CARGO_TARGET_DIR=${CARGO_TARGET_DIR}"
)

# Python dependencies target
add_custom_target(python_dependencies ALL
    COMMAND ${Python3_EXECUTABLE} -m pip install -r ${CMAKE_SOURCE_DIR}/promptveil/python/requirements.txt
    COMMENT "Installing Python dependencies"
)

# Install main package (includes core)
add_custom_target(python_package ALL
    COMMAND ${CMAKE_COMMAND} -E env 
        PROMPTVEIL_JULIA_PATH=${JULIA_BUILD_PATH}
        PROMPTVEIL_RUST_PATH=${CMAKE_BINARY_DIR}
        ${Python3_EXECUTABLE} -m pip install -e .
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/promptveil/python
)

# Dependencies
add_dependencies(python_package python_dependencies)
add_dependencies(python_package rust_security rust_format julia_core)

# Platform-specific Rust library extension
if(WIN32)
    set(RUST_LIB_EXT ".dll")
    set(RUST_LIB_PREFIX "")
elseif(APPLE)
    set(RUST_LIB_EXT ".dylib")
    set(RUST_LIB_PREFIX "lib")
else()
    set(RUST_LIB_EXT ".so")
    set(RUST_LIB_PREFIX "lib")
endif()

# Copy Rust library and setup Python package
add_custom_command(
    TARGET python_package POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CARGO_TARGET_DIR}/release/${RUST_LIB_PREFIX}promptveil_core${RUST_LIB_EXT}
        ${CMAKE_SOURCE_DIR}/promptveil/python/promptveil/core/
    COMMENT "Copying Rust library to Python package source"
)

# Copy to site-packages
if(WIN32)
    set(SITE_PACKAGES_PATH "${CMAKE_SOURCE_DIR}/build/venv/Lib/site-packages/promptveil_core")
else()
    set(SITE_PACKAGES_PATH "${CMAKE_SOURCE_DIR}/build/venv/lib/python${Python3_VERSION_MAJOR}.${Python3_VERSION_MINOR}/site-packages/promptveil_core")
endif()

add_custom_command(
    TARGET python_package POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory ${SITE_PACKAGES_PATH}
    COMMAND ${CMAKE_COMMAND} -E copy
        ${CARGO_TARGET_DIR}/release/${RUST_LIB_PREFIX}promptveil_core${RUST_LIB_EXT}
        ${SITE_PACKAGES_PATH}/
    COMMENT "Copying Rust library to site-packages"
)

# Generate __init__.py with correct imports
file(WRITE ${SITE_PACKAGES_PATH}/__init__.py
    "import importlib.util\n"
    "import sys\n\n"
    "# Get the module spec for the platform-specific extension\n"
    "if importlib.util.find_spec(\"promptveil_core.promptveil_core\"):\n"
    "    from .promptveil_core import *\n"
    "    __doc__ = promptveil_core.__doc__\n"
    "    if hasattr(promptveil_core, \"__all__\"):\n"
    "        __all__ = promptveil_core.__all__\n"
)

# Tests
if(BUILD_TESTS)
    enable_testing()
    
    # Julia tests
    add_test(
        NAME julia_tests
        COMMAND ${Julia_EXECUTABLE} --project=${JULIA_PROJECT_PATH} -e "using Pkg; Pkg.test()"
        WORKING_DIRECTORY ${JULIA_PROJECT_PATH}
    )
    
    # Rust tests
    add_test(
        NAME rust_security_tests
        COMMAND ${Rust_CARGO} test
        WORKING_DIRECTORY ${RUST_SECURITY_PATH}
    )
    
    add_test(
        NAME rust_format_tests
        COMMAND ${Rust_CARGO} test
        WORKING_DIRECTORY ${RUST_FORMAT_PATH}
    )
    
    # Python tests with pytest
    add_test(
        NAME python_tests
        COMMAND ${CMAKE_COMMAND} -E env
            PROMPTVEIL_JULIA_PATH=${JULIA_BUILD_PATH}
            PROMPTVEIL_RUST_PATH=${CMAKE_BINARY_DIR}
            ${Python3_EXECUTABLE} -m pytest
        WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/promptveil/python/tests
    )
endif()

# Documentation
if(BUILD_DOCS)
    find_package(Sphinx REQUIRED)
    add_custom_target(docs
        COMMAND ${SPHINX_EXECUTABLE} -b html
        ${CMAKE_SOURCE_DIR}/docs/source
        ${CMAKE_BINARY_DIR}/docs/html
    )
endif()

# Examples
if(BUILD_EXAMPLES)
    add_subdirectory(examples)
endif()

# Installation
install(
    FILES ${JULIA_BUILD_PATH}/PromptVeilCore${CMAKE_SHARED_LIBRARY_SUFFIX}
    DESTINATION lib
)

install(
    DIRECTORY promptveil/python/promptveil
    DESTINATION lib/python${Python3_VERSION_MAJOR}.${Python3_VERSION_MINOR}/site-packages
)

# Platform-specific configurations
if(WIN32)
    # Windows-specific settings
    set(CMAKE_WINDOWS_EXPORT_ALL_SYMBOLS ON)
    set(CMAKE_MSVC_RUNTIME_LIBRARY "MultiThreaded$<$<CONFIG:Debug>:Debug>")
elseif(APPLE)
    # macOS-specific settings
    set(CMAKE_MACOSX_RPATH ON)
    set(CMAKE_INSTALL_RPATH "@loader_path/../lib")
else()
    # Linux-specific settings
    set(CMAKE_INSTALL_RPATH "$ORIGIN/../lib")
endif() 

# Add Julia core target
add_custom_target(julia_core ALL
    COMMAND ${Julia_EXECUTABLE} --project=${JULIA_PROJECT_PATH} -e "include(\"build.jl\")"
    WORKING_DIRECTORY ${JULIA_PROJECT_PATH}
    COMMENT "Building Julia core library"
)

# Platform-specific library names
if(WIN32)
    set(JULIA_LIB_NAME "PromptVeilCore.dll")
    set(JULIA_LIB_EXTRA "PromptVeilCore.lib")
elseif(APPLE)
    set(JULIA_LIB_NAME "PromptVeilCore.dylib")
else()
    set(JULIA_LIB_NAME "PromptVeilCore.so")
endif()

# Create build directory if it doesn't exist
add_custom_command(
    TARGET julia_core PRE_BUILD
    COMMAND ${CMAKE_COMMAND} -E make_directory ${JULIA_BUILD_PATH}
)

# Copy Julia library to build directory
add_custom_command(
    TARGET julia_core POST_BUILD
    COMMAND ${CMAKE_COMMAND} -E copy
        ${JULIA_PROJECT_PATH}/${JULIA_LIB_NAME}
        ${JULIA_BUILD_PATH}/${JULIA_LIB_NAME}
    COMMENT "Copying Julia library to build directory"
)

# Copy extra files on Windows
if(WIN32)
    add_custom_command(
        TARGET julia_core POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
            ${JULIA_PROJECT_PATH}/${JULIA_LIB_EXTRA}
            ${JULIA_BUILD_PATH}/${JULIA_LIB_EXTRA}
        COMMENT "Copying Julia extra files"
    )
endif() 