#!/bin/bash

# Script parameters
FORCE_RUST_REBUILD=false
FORCE_JULIA_REBUILD=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --force-rust-rebuild)
            FORCE_RUST_REBUILD=true
            shift
            ;;
        --force-julia-rebuild)
            FORCE_JULIA_REBUILD=true
            shift
            ;;
        *)
            echo "Unknown parameter: $1"
            exit 1
            ;;
    esac
done

# Function to print timestamped messages
write_timestamped_message() {
    local message="$1"
    local color="$2"
    local timestamp=$(date '+%H:%M:%S.%3N')
    
    # ANSI color codes
    local reset='\033[0m'
    local cyan='\033[0;36m'
    local yellow='\033[0;33m'
    local red='\033[0;31m'
    local green='\033[0;32m'
    
    case $color in
        "cyan") color=$cyan ;;
        "yellow") color=$yellow ;;
        "red") color=$red ;;
        "green") color=$green ;;
        *) color=$reset ;;
    esac
    
    echo -e "[$timestamp] ${color}${message}${reset}"
}

start_time=$(date +%s)

# Check prerequisites
write_timestamped_message "Checking prerequisites..." "cyan"
./check_prereqs.sh
PREREQS_STATUS=$?

# If prerequisites check failed, try to install missing dependencies
if [ $PREREQS_STATUS -ne 0 ]; then
    if [ "$(uname)" = "Linux" ]; then
        write_timestamped_message "Some prerequisites are missing. Attempting to install..." "yellow"
        if [ -f "install_deps.sh" ]; then
            chmod +x install_deps.sh
            ./install_deps.sh
            if [ $? -ne 0 ]; then
                write_timestamped_message "Error: Failed to install dependencies" "red"
                exit 1
            fi
            # Check prerequisites again after installation
            ./check_prereqs.sh
            if [ $? -ne 0 ]; then
                write_timestamped_message "Error: Some prerequisites are still missing after installation" "red"
                exit 1
            fi
        else
            write_timestamped_message "Error: install_deps.sh not found" "red"
            exit 1
        fi
    else
        write_timestamped_message "Error: Automatic dependency installation is only supported on Linux" "red"
        write_timestamped_message "Please install missing dependencies manually" "red"
        exit 1
    fi
fi

write_timestamped_message "Starting PromptVeil build..." "cyan"

# Set environment variables
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
export JULIA_DIR=$(dirname $(dirname $(which julia)))
export PROMPTVEIL_CORE_DIR="$SCRIPT_DIR/promptveil/core/compression"

write_timestamped_message "Setting JULIA_DIR to: $JULIA_DIR" "yellow"
write_timestamped_message "Setting PROMPTVEIL_CORE_DIR to: $PROMPTVEIL_CORE_DIR" "yellow"

# 1. Julia Build Stage
write_timestamped_message "=== Julia Build Stage ===" "cyan"

# Clean Julia artifacts if forcing rebuild
if [ "$FORCE_JULIA_REBUILD" = true ]; then
    write_timestamped_message "Forcing Julia rebuild - cleaning artifacts..." "yellow"
    artifacts=(
        "PromptVeilCore.def"
        "PromptVeilCore.dll"
        "PromptVeilCore.so"
        "PromptVeilCore.dylib"
        "PromptVeilCore.exp"
        "PromptVeilCore.lib"
    )
    
    for artifact in "${artifacts[@]}"; do
        if [ -f "$PROMPTVEIL_CORE_DIR/$artifact" ]; then
            rm "$PROMPTVEIL_CORE_DIR/$artifact"
            write_timestamped_message "Removed $artifact" "yellow"
        fi
    done
fi

# Check if we need to rebuild Julia module
JULIA_LIB_PATH="promptveil/core/compression/PromptVeilCore"
JULIA_CACHE_PATH="build/julia_cache"
NEED_JULIA_BUILD=true

if [ -d "$JULIA_CACHE_PATH" ] && [ "$FORCE_JULIA_REBUILD" = false ]; then
    # Calculate current hash
    CURRENT_HASH=$(find promptveil/core/compression/TokenCompression.jl/src -type f -exec sha256sum {} \; | sort | sha256sum | cut -d' ' -f1)
    
    if [ -f "$JULIA_CACHE_PATH/hash.txt" ]; then
        CACHED_HASH=$(cat "$JULIA_CACHE_PATH/hash.txt")
        if [ "$CACHED_HASH" = "$CURRENT_HASH" ]; then
            write_timestamped_message "Julia build is up to date, skipping..." "green"
            NEED_JULIA_BUILD=false
            
            # Ensure library is in place
            if [ ! -f "${JULIA_LIB_PATH}.$(get_lib_extension)" ]; then
                write_timestamped_message "Restoring library from cache..." "yellow"
                cp "$JULIA_CACHE_PATH/PromptVeilCore.$(get_lib_extension)" "${JULIA_LIB_PATH}.$(get_lib_extension)"
            fi
        else
            write_timestamped_message "Source files changed, rebuilding Julia module..." "yellow"
        fi
    fi
fi

if [ "$NEED_JULIA_BUILD" = true ]; then
    # Build Julia module
    write_timestamped_message "Building Julia module..." "yellow"
    cd promptveil/core/compression
    julia --project=. setup.jl
    if [ $? -ne 0 ]; then
        write_timestamped_message "Error: Julia setup failed" "red"
        cd "$SCRIPT_DIR"
        exit 1
    fi

    julia --project=. build.jl
    if [ $? -ne 0 ]; then
        write_timestamped_message "Error: Julia build failed" "red"
        cd "$SCRIPT_DIR"
        exit 1
    fi
    cd "$SCRIPT_DIR"
    
    # Cache the new build
    mkdir -p "$JULIA_CACHE_PATH"
    find promptveil/core/compression/TokenCompression.jl/src -type f -exec sha256sum {} \; | sort | sha256sum | cut -d' ' -f1 > "$JULIA_CACHE_PATH/hash.txt"
    cp "${JULIA_LIB_PATH}.$(get_lib_extension)" "$JULIA_CACHE_PATH/"
fi

# 2. Python Environment Setup Stage
write_timestamped_message "=== Python Environment Setup Stage ===" "cyan"

# Create build directory
mkdir -p build

# Clean previous build but preserve venv and cargo cache
write_timestamped_message "Cleaning previous build..." "yellow"
if [ -d "build" ]; then
    # Save Cargo cache if it exists and we're not forcing a rebuild
    if [ -d "build/cargo_target" ] && [ "$FORCE_RUST_REBUILD" = false ]; then
        write_timestamped_message "Preserving Cargo cache..." "yellow"
        rm -rf cargo_target_backup
        mv build/cargo_target cargo_target_backup
    elif [ "$FORCE_RUST_REBUILD" = true ]; then
        write_timestamped_message "Forcing Rust rebuild - clearing Cargo cache..." "yellow"
    fi

    # Save venv if it exists
    if [ -d "build/venv" ]; then
        write_timestamped_message "Preserving virtual environment..." "yellow"
        rm -rf venv_backup
        mv build/venv venv_backup
    fi
    
    # Clean build directory but preserve specific folders
    find build -mindepth 1 -maxdepth 1 ! -name venv ! -name cargo_target -exec rm -rf {} +
    
    # Restore Cargo cache if not forcing rebuild
    if [ -d "cargo_target_backup" ] && [ "$FORCE_RUST_REBUILD" = false ]; then
        write_timestamped_message "Restoring Cargo cache..." "yellow"
        mv cargo_target_backup build/cargo_target
    fi

    # Restore venv
    if [ -d "venv_backup" ]; then
        write_timestamped_message "Restoring virtual environment..." "yellow"
        mv venv_backup build/venv
    fi
fi

# Create and setup Python virtual environment
write_timestamped_message "Setting up Python virtual environment..." "yellow"
VENV_PATH="build/venv"
if [ ! -d "$VENV_PATH" ]; then
    write_timestamped_message "Creating new virtual environment..." "yellow"
    python3 -m venv "$VENV_PATH"
fi

# Verify python exists in the venv
PYTHON_EXE="$VENV_PATH/bin/python3"
if [ ! -f "$PYTHON_EXE" ]; then
    write_timestamped_message "Error: Python executable not found in virtual environment at: $PYTHON_EXE" "red"
    exit 1
fi

# Activate venv
write_timestamped_message "Activating virtual environment..." "yellow"
source "$VENV_PATH/bin/activate"

# Install pip if not present
write_timestamped_message "Checking pip installation..." "yellow"
if ! $PYTHON_EXE -c "import pip" 2>/dev/null; then
    write_timestamped_message "pip not found in virtual environment, installing..." "yellow"
    curl -sSL https://bootstrap.pypa.io/get-pip.py -o get-pip.py
    $PYTHON_EXE get-pip.py
    rm get-pip.py
    if [ $? -ne 0 ]; then
        write_timestamped_message "Error: Failed to install pip" "red"
        exit 1
    fi
fi

# Install base dependencies
write_timestamped_message "Installing base dependencies..." "yellow"
$PYTHON_EXE -m pip install --upgrade pip setuptools wheel
if [ $? -ne 0 ]; then
    write_timestamped_message "Error: Failed to install base dependencies" "red"
    exit 1
fi

# 3. Rust Build Stage
write_timestamped_message "=== Rust Build Stage ===" "cyan"

# Set Python environment variables for Rust build
export PYO3_PYTHON="$PYTHON_EXE"
export PYO3_ENVIRONMENT_SIGNATURE="python-3.10-pyo3-0.20.3"
write_timestamped_message "Setting PYO3_PYTHON to: $PYO3_PYTHON" "yellow"
write_timestamped_message "Setting PYO3_ENVIRONMENT_SIGNATURE to: $PYO3_ENVIRONMENT_SIGNATURE" "yellow"

# Before CMake configuration, ensure Julia build directory exists and copy artifacts
JULIA_BUILD_DIR="build/julia_build"
mkdir -p "$JULIA_BUILD_DIR"

# Copy all Julia artifacts
for artifact in "PromptVeilCore.$(get_lib_extension)" "PromptVeilCore.def" "PromptVeilCore.exp" "PromptVeilCore.lib"; do
    if [ -f "$PROMPTVEIL_CORE_DIR/$artifact" ]; then
        write_timestamped_message "Copying Julia artifact: $artifact" "yellow"
        cp "$PROMPTVEIL_CORE_DIR/$artifact" "$JULIA_BUILD_DIR/"
    else
        write_timestamped_message "Warning: Julia artifact not found: $artifact" "yellow"
    fi
done

# Get system resources
NUM_CORES=$(nproc)
TOTAL_MEM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
TOTAL_MEM_GB=$((TOTAL_MEM_KB / 1024 / 1024))

# Calculate optimal number of jobs based on available resources
# Use 75% of available cores and ensure we leave some memory per job
OPTIMAL_JOBS=$((NUM_CORES * 3 / 4))
MEM_PER_JOB=2  # GB per job
MAX_JOBS_BY_MEM=$((TOTAL_MEM_GB / MEM_PER_JOB))

# Take the minimum between CPU-based and memory-based job count
if [ $MAX_JOBS_BY_MEM -lt $OPTIMAL_JOBS ]; then
    NUM_JOBS=$MAX_JOBS_BY_MEM
else
    NUM_JOBS=$OPTIMAL_JOBS
fi

write_timestamped_message "System resources:" "yellow"
write_timestamped_message "  - CPU cores: $NUM_CORES" "yellow"
write_timestamped_message "  - Memory: ${TOTAL_MEM_GB}GB" "yellow"
write_timestamped_message "Using $NUM_JOBS parallel jobs for compilation" "yellow"

# Set environment variables for build optimization
export MAKEFLAGS="-j$NUM_JOBS"
export CARGO_BUILD_JOBS="$NUM_JOBS"
export CMAKE_BUILD_PARALLEL_LEVEL="$NUM_JOBS"
export JULIA_NUM_THREADS="$NUM_JOBS"

# Configure CMake with parallel build
cd build
cmake .. \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_BUILD_PARALLEL_LEVEL="$NUM_JOBS" \
    -DCMAKE_C_FLAGS="-O3 -march=native" \
    -DCMAKE_CXX_FLAGS="-O3 -march=native"

# Build with all cores
cmake --build . --parallel "$NUM_JOBS"

cd "$SCRIPT_DIR"

# 4. Python Package Installation Stage
write_timestamped_message "=== Python Package Installation Stage ===" "cyan"

write_timestamped_message "Installing Python package..." "yellow"

# Install requirements first
write_timestamped_message "Installing requirements from: $SCRIPT_DIR/promptveil/python/requirements.txt" "yellow"
$PYTHON_EXE -m pip install -r "$SCRIPT_DIR/promptveil/python/requirements.txt"
if [ $? -ne 0 ]; then
    write_timestamped_message "Error: Failed to install Python requirements" "red"
    exit 1
fi

# Set environment variables for the package installation
export PROMPTVEIL_JULIA_PATH="$SCRIPT_DIR/build/julia_build"
export PROMPTVEIL_RUST_PATH="$SCRIPT_DIR/build"

# Install the package in development mode
write_timestamped_message "Installing PromptVeil package in development mode..." "yellow"
cd "$SCRIPT_DIR/promptveil/python"
$PYTHON_EXE -m pip install -e .
if [ $? -ne 0 ]; then
    write_timestamped_message "Error: Failed to install PromptVeil package" "red"
    cd "$SCRIPT_DIR"
    exit 1
fi

cd "$SCRIPT_DIR"

end_time=$(date +%s)
duration=$((end_time - start_time))
write_timestamped_message "Build completed successfully in $(date -u -d @${duration} '+%H:%M:%S')!" "green"
write_timestamped_message "You can now use the PromptVeil package from Python." "green"

# Helper function to get library extension based on OS
get_lib_extension() {
    case "$(uname)" in
        "Darwin") echo "dylib" ;;
        "Linux") echo "so" ;;
        "MINGW"*|"MSYS"*|"CYGWIN"*) echo "dll" ;;
        *) 
            if [ -f "/proc/version" ] && grep -qi microsoft "/proc/version"; then
                echo "dll"  # WSL
            else
                echo "so"  # Default to .so for unknown Unix-like systems
            fi
            ;;
    esac
} 