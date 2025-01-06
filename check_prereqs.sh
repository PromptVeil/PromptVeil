#!/bin/bash

# Function to check if a command exists
check_command() {
    local cmd=$1
    local name=$2
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $name is not installed or not in PATH"
        return 1
    fi
    return 0
}

# Function to check Julia version
check_julia_version() {
    local version=$(julia --version | cut -d' ' -f3)
    if [[ "$version" < "1.6.0" ]]; then
        echo "Error: Julia version must be >= 1.6.0 (found $version)"
        return 1
    fi
    return 0
}

# Function to check Python version
check_python_version() {
    local version=$(python3 --version 2>&1 | cut -d' ' -f2)
    if [[ "$version" < "3.8.0" ]]; then
        echo "Error: Python version must be >= 3.8.0 (found $version)"
        return 1
    fi
    return 0
}

# Function to check Rust version
check_rust_version() {
    local version=$(rustc --version | cut -d' ' -f2)
    if [[ "$version" < "1.70.0" ]]; then
        echo "Error: Rust version must be >= 1.70.0 (found $version)"
        return 1
    fi
    return 0
}

# Check CMake
echo "Checking CMake..."
check_command cmake "CMake" || exit 1
cmake_version=$(cmake --version | head -n1 | cut -d' ' -f3)
echo "Found CMake version $cmake_version"

# Check Python
echo "Checking Python..."
check_command python3 "Python" || exit 1
python3 --version
check_python_version || exit 1

# Check Rust
echo "Checking Rust..."
check_command rustc "Rust" || exit 1
rustc --version
check_rust_version || exit 1

# Check Julia
echo "Checking Julia..."
check_command julia "Julia" || exit 1
julia --version
check_julia_version || exit 1

# Check for build tools
echo "Checking build tools..."

# Check for GCC/Clang
if command -v gcc &> /dev/null; then
    echo "Found GCC: $(gcc --version | head -n1)"
elif command -v clang &> /dev/null; then
    echo "Found Clang: $(clang --version | head -n1)"
else
    echo "Error: Neither GCC nor Clang found"
    exit 1
fi

# Check for Make
check_command make "Make" || exit 1
echo "Found Make: $(make --version | head -n1)"

# All checks passed
echo "All prerequisites found successfully!"
exit 0 