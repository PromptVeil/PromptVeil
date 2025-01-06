# FindPromptVeilCore.cmake
# Finds the PromptVeilCore library compiled by Julia
#
# This module defines:
# PromptVeilCore_FOUND - If the library was found
# PromptVeilCore_LIBRARY - Full path to the library
# PromptVeilCore_INCLUDE_DIR - Header directory (if necessary)

# Determine library extension based on the operating system
if(WIN32)
    set(LIB_EXTENSION "dll")
elseif(APPLE)
    set(LIB_EXTENSION "dylib")
else()
    set(LIB_EXTENSION "so")
endif()

# Search for the library in the Julia build directory
find_file(PromptVeilCore_LIBRARY
    NAMES 
        "PromptVeilCore.${LIB_EXTENSION}"
        "libPromptVeilCore.${LIB_EXTENSION}"
    PATHS
        "${CMAKE_SOURCE_DIR}/promptveil/core/compression"
        "${CMAKE_BINARY_DIR}/julia_build"
        "${JULIA_PROJECT_PATH}"
        "${JULIA_BUILD_PATH}"
    PATH_SUFFIXES
        .
        lib
        bin
    NO_DEFAULT_PATH
    DOC "Path to PromptVeilCore library"
)

message(STATUS "Searching for PromptVeilCore in:")
message(STATUS "  ${CMAKE_SOURCE_DIR}/promptveil/core/compression")
message(STATUS "  ${CMAKE_BINARY_DIR}/julia_build")
message(STATUS "  ${JULIA_PROJECT_PATH}")
message(STATUS "  ${JULIA_BUILD_PATH}")

if(PromptVeilCore_LIBRARY)
    message(STATUS "Found PromptVeilCore: ${PromptVeilCore_LIBRARY}")
else()
    message(STATUS "PromptVeilCore not found")
endif()

# If the library is found, mark it as found
include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(PromptVeilCore
    REQUIRED_VARS
        PromptVeilCore_LIBRARY
)

# If found, create a variable with the library directory
if(PromptVeilCore_FOUND)
    get_filename_component(PromptVeilCore_DIR ${PromptVeilCore_LIBRARY} DIRECTORY)
    mark_as_advanced(PromptVeilCore_LIBRARY PromptVeilCore_DIR)
endif() 