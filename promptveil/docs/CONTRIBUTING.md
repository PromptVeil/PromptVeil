# Contributing to PromptVeil

We love your input! We want to make contributing to PromptVeil as easy and transparent as possible, whether it's:

- Reporting a bug
- Discussing the current state of the code
- Submitting a fix
- Proposing new features
- Becoming a maintainer

## Development Process

We use GitHub to host code, to track issues and feature requests, as well as accept pull requests.

1. Fork the repo and create your branch from `main`
2. Name your branch based on the type of change:
   - `feature/description`
   - `fix/description`
   - `docs/description`
   - `release/vX.Y.Z`
3. If you've added code that should be tested, add tests
4. If you've changed APIs, update the documentation
5. Ensure the test suite passes
6. Make sure your code lints
7. Open a Pull Request to `main`

Note: Direct pushes to `main` are not allowed. All changes must go through Pull Requests with required reviews.

## Branch Protection Rules

The `main` branch is protected with the following rules:
- No direct pushes allowed
- Requires at least one approved review
- Must pass all CI checks
- Must be up to date with base branch
- Linear history (no merge commits)

## Release Process

We use semantic versioning (MAJOR.MINOR.PATCH):
1. MAJOR version for incompatible API changes
2. MINOR version for new functionality in a backward compatible manner
3. PATCH version for backward compatible bug fixes

To create a new release:
1. Create a release branch:
   ```bash
   git checkout -b release/vX.Y.Z
   ```
2. Update version in `promptveil/python/setup.py`
3. Update CHANGELOG.md
4. Create a Pull Request to `main`
5. After PR is approved and merged, create and push tag:
   ```bash
   git checkout main
   git pull origin main
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
6. The GitHub Actions workflow will:
   - Run all tests across platforms
   - Build the package
   - Publish to PyPI (only for tagged releases)

## Code Structure

```
promptveil/
├── core/
│   ├── compression/    # Julia core (token compression)
│   ├── security/      # Rust security layer
│   └── format/        # Rust .pveil format
├── python/           # Python API
└── docs/            # Documentation
```

## Library Architecture and Linking

The project uses a multi-layer architecture with Julia, Rust, and Python:

1. **Julia Core Library (~468MB)**
   - Implements the core token compression algorithms
   - Compiled to a shared library (`PromptVeilCore.dll`/`.so`/`.dylib`)
   - The large size is due to embedded ML models and Julia runtime

2. **Platform-Specific Linking**
   
   Windows:
   - Uses `.lib` file for compile-time linking
   - Uses `DELAYLOAD` for runtime DLL loading
   - Copies DLL to output directory
   - No symbolic links needed

   Linux:
   - Uses `.so` for linking
   - Copies `.so` to output directory
   - Uses `rpath` with `$ORIGIN` for library lookup
   - No symbolic links needed

   macOS:
   - Uses `.dylib` for linking (similar to Linux)
   - Implementation details pending verification

3. **Build Process**
   - Julia compiles `PromptVeilCore` shared library
   - Rust security layer links against this library
   - Python bindings link against the Rust library
   - Build system ensures correct library placement and linking

## Development Setup

1. **Clone your fork**
   ```bash
   git clone https://github.com/yourusername/promptveil.git
   ```

2. **Install development dependencies**
   ```bash
   # Python dependencies
   pip install -r requirements-dev.txt
   
   # Julia dependencies
   julia --project=. -e 'using Pkg; Pkg.develop(path="core/compression")'
   
   # Rust dependencies
   cd core/security && cargo build
   cd ../format && cargo build
   ```

## Testing

We use multiple test suites:

1. **Julia Tests**
   ```bash
   cd core/compression
   julia --project=. -e 'using Pkg; Pkg.test()'
   ```

2. **Rust Tests**
   ```bash
   cd core/security
   cargo test
   cd ../format
   cargo test
   ```

3. **Python Tests**
   ```bash
   pytest python/tests
   ```

## Documentation

- Use docstrings for function and class documentation
- Update markdown docs in `/docs`
- Add examples for new features

## Code Style

### Julia
- Follow [Julia style guide](https://docs.julialang.org/en/v1/manual/style-guide/)
- Use meaningful variable names
- Add type annotations for public functions

### Rust
- Follow [Rust style guide](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing
- Run `clippy` for additional checks

### Python
- Follow PEP 8
- Use type hints
- Run `black` for formatting

## Pull Request Process

1. Update the README.md with details of changes if needed
2. Update the docs with details of changes if needed
3. The PR will be merged once you have the sign-off of two maintainers

## Any contributions you make will be under the MIT Software License
In short, when you submit code changes, your submissions are understood to be under the same [MIT License](http://choosealicense.com/licenses/mit/) that covers the project. Feel free to contact the maintainers if that's a concern.

## Report bugs using GitHub's [issue tracker](https://github.com/yourusername/promptveil/issues)
We use GitHub issues to track public bugs. Report a bug by [opening a new issue](https://github.com/yourusername/promptveil/issues/new).

## Write bug reports with detail, background, and sample code

**Great Bug Reports** tend to have:

- A quick summary and/or background
- Steps to reproduce
  - Be specific!
  - Give sample code if you can
- What you expected would happen
- What actually happens
- Notes (possibly including why you think this might be happening, or stuff you tried that didn't work)

## License
By contributing, you agree that your contributions will be licensed under its MIT License. 