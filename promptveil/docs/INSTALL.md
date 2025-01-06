# Installation Guide

## Prerequisites

### Required Software

1. **CMake**
   - Version 3.15 or later
   - Required for build system

2. **Python**
   - Version 3.7 or later
   - pip package manager

3. **Julia**
   - Version 1.6 or later
   - Add to system PATH

4. **Rust**
   - Latest stable version
   - cargo package manager

### Platform-Specific Requirements

#### Windows
1. Visual Studio Build Tools or Visual Studio Community
   - C++ build tools
   - Windows SDK
   - Ensure delayimp.lib is available (part of Visual Studio)
2. LLVM (optional, for better optimizations)
   ```powershell
   # Set environment variable for LLVM
   $env:LIBCLANG_PATH="C:\Program Files\LLVM\bin"
   ```

#### Linux (Ubuntu/Debian)
```bash
# Install build essentials and CMake
sudo apt-get update
sudo apt-get install build-essential cmake
```

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install CMake via Homebrew
brew install cmake
```

## Installation Steps

### 1. Install from PyPI (Recommended)
```bash
pip install promptveil
```

### 2. Install from Source

1. **Clone the Repository**
   ```bash
   git clone https://github.com/yourusername/promptveil.git
   cd promptveil
   ```

2. **Configure with CMake**
   ```bash
   # Create build directory
   mkdir build && cd build

   # Configure
   cmake ..

   # Build
   cmake --build .

   # Install
   cmake --install .
   ```

   CMake Options:
   - `-DBUILD_TESTS=ON|OFF`: Enable/disable tests (default: ON)
   - `-DUSE_GPU=ON|OFF`: Enable/disable GPU acceleration (default: ON)
   - `-DCMAKE_BUILD_TYPE=Release|Debug`: Set build type

3. **Run Tests**
   ```bash
   # In build directory
   ctest --output-on-failure
   ```

## Verification

Test your installation:
```python
from promptveil import Conversation

# Should run without errors
conv = Conversation()
conv.add_message("user", "Test message")
conv.save("test.pveil")
```

## Troubleshooting

### Common Issues

1. **CMake Not Found**
   - Ensure CMake is installed: `cmake --version`
   - Add CMake to PATH if needed

2. **Julia Not Found**
   - Ensure Julia is in system PATH
   - Check Julia installation: `julia --version`

3. **Rust Build Failures**
   - Update Rust: `rustup update`
   - Clean build: `cargo clean`

4. **Windows-Specific**
   - LLVM path not set
   - Missing Visual Studio components

### Getting Help

- File issues on GitHub
- Check documentation
- Join our community Discord

## Updating

### PyPI Version
```bash
pip install --upgrade promptveil
```

### Source Installation
```bash
git pull
cd build
cmake ..
cmake --build .
cmake --install .
```

## Build System Architecture

### Component Dependencies

The build system manages several interdependent components:

1. **Julia Core (PromptVeilCore.dll)**
   - Built from Julia code
   - Provides token compression functionality
   - Located in `build/julia_build/`

2. **Rust Core (promptveil_core.dll)**
   - Built from Rust code with PyO3
   - Embeds and manages Julia Core DLL
   - Uses delay-loading for Julia DLL
   - Located in `build/cargo_target/release/`

3. **Python Package**
   - Imports Rust Core module
   - Provides high-level API
   - Located in `promptveil/python/`

### Build Process

1. **Automated Build Script**
   ```powershell
   # Normal build (uses cache if available)
   .\build.ps1

   # Force Rust components rebuild
   .\build.ps1 -ForceRustRebuild

   # Force Julia components rebuild
   .\build.ps1 -ForceJuliaRebuild

   # Force both Rust and Julia rebuild
   .\build.ps1 -ForceRustRebuild -ForceJuliaRebuild
   ```

   The build script uses caching by default to speed up subsequent builds:
   - Without parameters: Uses cached builds if available
   - `-ForceRustRebuild`: Forces rebuild of Rust components
   - `-ForceJuliaRebuild`: Forces rebuild of Julia components
   - Both flags can be combined to force a complete rebuild

2. **Build Stages**
   - Julia Build Stage: Compiles Julia code into DLL
   - Python Environment Setup: Creates and configures venv
   - Rust Build Stage: Compiles Rust code with Julia integration
   - Python Package Installation: Installs the package in development mode

### DLL Management

The framework uses a sophisticated DLL management approach:

1. **Julia DLL Integration**
   - Julia's `PromptVeilCore.dll` is embedded in Rust's build
   - Uses delay-loading to manage DLL dependencies
   - No need for PATH modifications

2. **Rust DLL Distribution**
   - Single DLL distribution (`promptveil_core.dll`)
   - Contains all native code dependencies
   - Automatically manages Julia DLL loading

3. **Installation Verification**
   ```python
   # Test both Rust and Julia integration
   from promptveil.core import promptveil_core
   from promptveil import Conversation

   # Should run without errors
   conv = Conversation()
   ``` 