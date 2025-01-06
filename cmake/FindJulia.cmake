# FindJulia.cmake
# Finds Julia executable and libraries
#
# This will define:
# Julia_FOUND - True if Julia was found
# Julia_EXECUTABLE - Path to Julia executable
# Julia_INCLUDE_DIRS - Include directories
# Julia_LIBRARY_DIRS - Library directories
# Julia_LIBRARIES - Libraries to link against

# Find Julia executable
find_program(Julia_EXECUTABLE
    NAMES julia julia.exe
    DOC "Julia executable"
)

if(Julia_EXECUTABLE)
    # Get Julia version
    execute_process(
        COMMAND ${Julia_EXECUTABLE} --version
        OUTPUT_VARIABLE Julia_VERSION_STRING
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    # Get Julia paths
    execute_process(
        COMMAND ${Julia_EXECUTABLE} -e "print(dirname(dirname(Sys.BINDIR)))"
        OUTPUT_VARIABLE Julia_DIR
        OUTPUT_STRIP_TRAILING_WHITESPACE
    )

    set(Julia_INCLUDE_DIRS "${Julia_DIR}/include/julia")
    set(Julia_LIBRARY_DIRS "${Julia_DIR}/lib")
    
    if(WIN32)
        set(Julia_LIBRARIES "${Julia_LIBRARY_DIRS}/libjulia.dll.a")
    elseif(APPLE)
        set(Julia_LIBRARIES "${Julia_LIBRARY_DIRS}/libjulia.dylib")
    else()
        set(Julia_LIBRARIES "${Julia_LIBRARY_DIRS}/libjulia.so")
    endif()
endif()

include(FindPackageHandleStandardArgs)
find_package_handle_standard_args(Julia
    REQUIRED_VARS
        Julia_EXECUTABLE
        Julia_INCLUDE_DIRS
        Julia_LIBRARY_DIRS
        Julia_LIBRARIES
    VERSION_VAR
        Julia_VERSION_STRING
) 