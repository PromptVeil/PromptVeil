cmake_minimum_required(VERSION 3.15)
project(PromptVeil VERSION 0.1.0 LANGUAGES CXX)

# Add cmake modules path
list(APPEND CMAKE_MODULE_PATH "${CMAKE_CURRENT_SOURCE_DIR}/cmake")

# Options
option(BUILD_TESTS "Build tests" ON)
option(USE_GPU "Enable GPU acceleration" ON)
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

# Configure Rust build cache
set(CARGO_TARGET_DIR "${CMAKE_BINARY_DIR}/cargo_target")
set(ENV{CARGO_TARGET_DIR} "${CARGO_TARGET_DIR}")

# Environment variables for PyO3
set(ENV{PYTHON_SYS_EXECUTABLE} ${Python3_EXECUTABLE})
set(ENV{PYO3_PYTHON} ${Python3_EXECUTABLE})

# Create Julia build directory
file(MAKE_DIRECTORY ${JULIA_BUILD_PATH})

# Handle platform-specific library extensions and import libraries
if(WIN32)
    set(JULIA_ARTIFACTS
        "${JULIA_PROJECT_PATH}/PromptVeilCore.dll"
        "${JULIA_PROJECT_PATH}/PromptVeilCore.lib"
        "${JULIA_PROJECT_PATH}/PromptVeilCore.def"
        "${JULIA_PROJECT_PATH}/PromptVeilCore.exp"
    )
    
    # Copy all Julia artifacts to build directory
    foreach(ARTIFACT ${JULIA_ARTIFACTS})
        get_filename_component(ARTIFACT_NAME ${ARTIFACT} NAME)
        add_custom_command(
            OUTPUT ${JULIA_BUILD_PATH}/${ARTIFACT_NAME}
            COMMAND ${CMAKE_COMMAND} -E copy
                ${ARTIFACT}
                ${JULIA_BUILD_PATH}/${ARTIFACT_NAME}
            DEPENDS ${ARTIFACT}
            COMMENT "Copying Julia artifact: ${ARTIFACT_NAME}"
        )
        list(APPEND JULIA_BUILD_OUTPUTS ${JULIA_BUILD_PATH}/${ARTIFACT_NAME})
    endforeach()
    
    # Custom target for all Julia artifacts
    add_custom_target(julia_core ALL
        DEPENDS ${JULIA_BUILD_OUTPUTS}
    )

    # Set library paths for Rust targets
    set(ENV{LIB} "${JULIA_BUILD_PATH};$ENV{LIB}")
    link_directories(${JULIA_BUILD_PATH})
else()
    # Unix systems (Linux/macOS)
    set(LIB_EXTENSION ${CMAKE_SHARED_LIBRARY_SUFFIX})
    add_custom_command(
        OUTPUT ${JULIA_BUILD_PATH}/PromptVeilCore${LIB_EXTENSION}
        COMMAND ${CMAKE_COMMAND} -E copy
            ${JULIA_PROJECT_PATH}/PromptVeilCore${LIB_EXTENSION}
            ${JULIA_BUILD_PATH}/PromptVeilCore${LIB_EXTENSION}
        DEPENDS ${JULIA_PROJECT_PATH}/PromptVeilCore${LIB_EXTENSION}
        COMMENT "Copying Julia core module"
    )
    
    add_custom_target(julia_core ALL
        DEPENDS ${JULIA_BUILD_PATH}/PromptVeilCore${LIB_EXTENSION}
    )
endif()

# Find PromptVeilCore library
find_package(PromptVeilCore REQUIRED)

# Set environment variables for Rust
set(ENV{JULIA_DIR} "${JULIA_ROOT_DIR}")
set(ENV{PROMPTVEIL_CORE_DIR} "${CMAKE_SOURCE_DIR}/promptveil/core/compression")
set(ENV{RUST_BACKTRACE} "1")
set(ENV{CARGO_TARGET_DIR} "${CARGO_TARGET_DIR}")

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

# Python package with development mode
add_custom_target(python_dependencies
    COMMAND ${Python3_EXECUTABLE} -m pip install -r requirements.txt
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/promptveil/python
)

# Build and install core package first
add_custom_target(python_core_package
    COMMAND ${CMAKE_COMMAND} -E env 
        PROMPTVEIL_JULIA_PATH=${JULIA_BUILD_PATH}
        PROMPTVEIL_RUST_PATH=${CMAKE_BINARY_DIR}
        ${Python3_EXECUTABLE} -m pip install -e .
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/promptveil/core/python
)

# Then install main package
add_custom_target(python_package ALL
    COMMAND ${CMAKE_COMMAND} -E env 
        PROMPTVEIL_JULIA_PATH=${JULIA_BUILD_PATH}
        PROMPTVEIL_RUST_PATH=${CMAKE_BINARY_DIR}
        ${Python3_EXECUTABLE} -m pip install -e .
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/promptveil/python
)

# Dependencies
add_dependencies(python_core_package python_dependencies)
add_dependencies(python_package python_core_package)
add_dependencies(rust_security julia_core)
add_dependencies(rust_format julia_core)
add_dependencies(python_package rust_security rust_format)

# Copy Rust DLL to Python package directory
if(WIN32)
    add_custom_command(
        TARGET python_package POST_BUILD
        COMMAND ${CMAKE_COMMAND} -E copy
            ${CARGO_TARGET_DIR}/release/promptveil_core.dll
            ${CMAKE_SOURCE_DIR}/build/venv/Lib/site-packages/promptveil_core/
        COMMENT "Copying Rust DLL to Python package directory"
    )
    
    # Generate __init__.py with correct imports
    file(WRITE ${CMAKE_SOURCE_DIR}/build/venv/Lib/site-packages/promptveil_core/__init__.py
        "import importlib.util\n"
        "import sys\n\n"
        "# Get the module spec for the platform-specific extension\n"
        "if importlib.util.find_spec(\"promptveil_core.promptveil_core\"):\n"
        "    from .promptveil_core import *\n"
        "    __doc__ = promptveil_core.__doc__\n"
        "    if hasattr(promptveil_core, \"__all__\"):\n"
        "        __all__ = promptveil_core.__all__\n"
    )
endif()

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