# Script parameters
param(
    [switch]$ForceRustRebuild,
    [switch]$ForceJuliaRebuild
)

function Write-TimestampedMessage {
    param(
        [string]$Message,
        [System.ConsoleColor]$Color = [System.ConsoleColor]::White
    )
    $timestamp = Get-Date -Format "HH:mm:ss.fff"
    Write-Host "[$timestamp] $Message" -ForegroundColor $Color
}

$startTime = Get-Date

# Check prerequisites
Write-TimestampedMessage "Checking prerequisites..." -Color Cyan
.\check_prereqs.ps1
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Some prerequisites are missing." -Color Red
    exit 1
}

Write-TimestampedMessage "`nStarting PromptVeil build..." -Color Cyan

# Set JULIA_DIR environment variable
$juliaExe = (Get-Command julia).Source
$juliaDir = (Get-Item $juliaExe).Directory.Parent.FullName
$env:JULIA_DIR = $juliaDir
Write-TimestampedMessage "Setting JULIA_DIR to: $juliaDir" -Color Yellow

# Set PROMPTVEIL_CORE_DIR environment variable
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$promptVeilCoreDir = Join-Path $scriptDir "promptveil/core/compression"
$env:PROMPTVEIL_CORE_DIR = $promptVeilCoreDir
Write-TimestampedMessage "Setting PROMPTVEIL_CORE_DIR to: $promptVeilCoreDir" -Color Yellow

# 1. Julia Build Stage
Write-TimestampedMessage "`n=== Julia Build Stage ===" -Color Cyan

# Clean Julia artifacts if forcing rebuild
if ($ForceJuliaRebuild) {
    Write-TimestampedMessage "Forcing Julia rebuild - cleaning artifacts..." -Color Yellow
    $artifactsToRemove = @(
        "PromptVeilCore.def",
        "PromptVeilCore.dll",
        "PromptVeilCore.dll.bak",
        "PromptVeilCore.exp",
        "PromptVeilCore.lib"
    )
    
    foreach ($artifact in $artifactsToRemove) {
        $artifactPath = Join-Path $promptVeilCoreDir $artifact
        if (Test-Path $artifactPath) {
            Remove-Item $artifactPath -Force
            Write-TimestampedMessage "Removed $artifact" -Color Yellow
        }
    }
}

# Check if we need to rebuild Julia module
$juliaLibPath = "promptveil/core/compression/PromptVeilCore.dll"
$juliaImportLibPath = "promptveil/core/compression/PromptVeilCore.lib"
$juliaCachePath = "build/julia_cache"
$needJuliaBuild = $true

# Define required files based on OS
$isWindows = $PSVersionTable.Platform -eq $null -or $PSVersionTable.Platform -eq "Win32NT"
$isMacOS = $PSVersionTable.OS -like "*Darwin*"

$requiredFiles = if ($isWindows) {
    @(
        "promptveil/core/compression/PromptVeilCore.dll",
        "promptveil/core/compression/PromptVeilCore.lib",
        "promptveil/core/compression/PromptVeilCore.exp",
        "promptveil/core/compression/PromptVeilCore.def"
    )
} else {
    @("promptveil/core/compression/PromptVeilCore.$(if ($isMacOS) { 'dylib' } else { 'so' })")
}

# Skip cache check if forcing rebuild
if ($ForceJuliaRebuild) {
    Write-TimestampedMessage "Forcing Julia rebuild..." -Color Yellow
    $needJuliaBuild = $true
} else {
    # Check if all required files exist
    $missingFiles = $requiredFiles | Where-Object { -not (Test-Path $_) }
    if ($missingFiles) {
        Write-TimestampedMessage "Missing required files:" -Color Yellow
        $missingFiles | ForEach-Object { Write-TimestampedMessage "  - $_" -Color Yellow }
        $needJuliaBuild = $true
    } else {
        # Initialize cache with existing files if no cache exists
        if (-not (Test-Path $juliaCachePath)) {
            Write-TimestampedMessage "Initializing Julia cache with existing files..." -Color Yellow
            New-Item -ItemType Directory -Force -Path $juliaCachePath | Out-Null
            
            # Copy all required files to cache
            foreach ($file in $requiredFiles) {
                Copy-Item $file "$juliaCachePath/$(Split-Path -Leaf $file)" -Force
            }
            
            # Calculate and store hash of source files
            $sourceFiles = Get-ChildItem -Path "promptveil/core/compression/TokenCompression.jl/src" -File -Recurse
            $sourceHashes = $sourceFiles | ForEach-Object { Get-FileHash $_.FullName } | ForEach-Object { $_.Hash }
            $combinedHash = [string]::Join("", $sourceHashes)
            $combinedHash | Set-Content "$juliaCachePath/hash.txt"
            
            Write-TimestampedMessage "Cache initialized with all required files" -Color Green
            $needJuliaBuild = $false
        }
    }
}

if ($needJuliaBuild) {
    # Build Julia module
    Write-TimestampedMessage "`nBuilding Julia module..." -Color Yellow
    Set-Location promptveil/core/compression
    julia --project=. setup.jl
    if ($LASTEXITCODE -ne 0) {
        Write-TimestampedMessage "Error: Julia setup failed" -Color Red
        Set-Location $scriptDir
        exit 1
    }

    julia --project=. build.jl
    if ($LASTEXITCODE -ne 0) {
        Write-TimestampedMessage "Error: Julia build failed" -Color Red
        Set-Location $scriptDir
        exit 1
    }
    Set-Location $scriptDir
    
    # Cache the new build
    New-Item -ItemType Directory -Force -Path $juliaCachePath | Out-Null
    
    # Calculate hash without Join-String
    $sourceFiles = Get-ChildItem -Path "promptveil/core/compression/TokenCompression.jl/src" -File -Recurse
    $sourceHashes = $sourceFiles | ForEach-Object { Get-FileHash $_.FullName } | ForEach-Object { $_.Hash }
    $combinedHash = [string]::Concat($sourceHashes)
    Set-Content "$juliaCachePath/hash.txt" -Value $combinedHash
    
    Copy-Item $juliaLibPath "$juliaCachePath/PromptVeilCore.dll" -Force
}

# 2. Python Environment Setup Stage
Write-TimestampedMessage "`n=== Python Environment Setup Stage ===" -Color Cyan

# Create build directory if it doesn't exist
$buildDir = Join-Path $scriptDir "build"
New-Item -ItemType Directory -Force -Path $buildDir | Out-Null

# Clean previous build but preserve venv and cargo cache
Write-TimestampedMessage "Cleaning previous build..." -Color Yellow
if (Test-Path $buildDir) {
    # Save Cargo cache if it exists and we're not forcing a rebuild
    $cargoCache = Join-Path $buildDir "cargo_target"
    $cargoCacheBackup = "cargo_target_backup"
    if ((Test-Path $cargoCache) -and (-not $ForceRustRebuild)) {
        Write-TimestampedMessage "Preserving Cargo cache..." -Color Yellow
        if (Test-Path $cargoCacheBackup) {
            Remove-Item -Recurse -Force $cargoCacheBackup
        }
        Move-Item $cargoCache $cargoCacheBackup
    } elseif ($ForceRustRebuild) {
        Write-TimestampedMessage "Forcing Rust rebuild - clearing Cargo cache..." -Color Yellow
    }

    # Clean build directory but preserve specific folders
    Get-ChildItem -Path $buildDir -Exclude @("venv", "cargo_target") | Remove-Item -Recurse -Force
    
    # Restore Cargo cache if not forcing rebuild
    if ((Test-Path $cargoCacheBackup) -and (-not $ForceRustRebuild)) {
        Write-TimestampedMessage "Restoring Cargo cache..." -Color Yellow
        Move-Item $cargoCacheBackup $cargoCache
    }
}

# Create and setup Python virtual environment
Write-TimestampedMessage "Setting up Python virtual environment..." -Color Yellow
$venvPath = Join-Path $buildDir "venv"

# Remove existing venv if it exists or if any rebuild flag is set
if ((Test-Path $venvPath) -or $ForceRustRebuild -or $ForceJuliaRebuild) {
    Write-TimestampedMessage "Removing existing virtual environment..." -Color Yellow
    if (Test-Path $venvPath) {
        Remove-Item -Recurse -Force $venvPath
    }
}

# Create new venv
Write-TimestampedMessage "Creating new virtual environment..." -Color Yellow
python -m venv $venvPath
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Failed to create virtual environment" -Color Red
    exit 1
}

# Verify python.exe exists in the venv
$pythonExe = Join-Path $venvPath "Scripts/python.exe"
if (-not (Test-Path $pythonExe)) {
    Write-TimestampedMessage "Error: Python executable not found in virtual environment at: $pythonExe" -Color Red
    exit 1
}

# Activate venv
Write-TimestampedMessage "Activating virtual environment..." -Color Yellow
$activateScript = Join-Path $venvPath "Scripts/Activate.ps1"
if (-not (Test-Path $activateScript)) {
    Write-TimestampedMessage "Error: Activation script not found at: $activateScript" -Color Red
    exit 1
}

# Use & to invoke the activation script
& $activateScript

# Verify venv is activated
$currentPython = (Get-Command python).Source
if (-not $currentPython.StartsWith($venvPath)) {
    Write-TimestampedMessage "Error: Virtual environment not properly activated. Current Python: $currentPython" -Color Red
    exit 1
}

Write-TimestampedMessage "Virtual environment activated successfully" -Color Green

# Install pip if not present
Write-TimestampedMessage "Checking pip installation..." -Color Yellow
$pipResult = & $pythonExe -c "import pip" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "pip not found in virtual environment, installing..." -Color Yellow
    $getTempFile = Join-Path $env:TEMP "get-pip.py"
    Invoke-WebRequest "https://bootstrap.pypa.io/get-pip.py" -OutFile $getTempFile
    & $pythonExe $getTempFile
    Remove-Item $getTempFile
    if ($LASTEXITCODE -ne 0) {
        Write-TimestampedMessage "Error: Failed to install pip" -Color Red
        exit 1
    }
}

# Install base dependencies first
Write-TimestampedMessage "`nInstalling base dependencies..." -Color Yellow
& $pythonExe -m pip install --upgrade pip setuptools wheel
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Failed to install base dependencies" -Color Red
    exit 1
}

# 3. Rust Build Stage
Write-TimestampedMessage "`n=== Rust Build Stage ===" -Color Cyan

# Set Python environment variables for Rust build
$env:PYO3_PYTHON = $pythonExe
$env:PYO3_ENVIRONMENT_SIGNATURE = "python-3.10-pyo3-0.20.3"
Write-TimestampedMessage "Setting PYO3_PYTHON to: $env:PYO3_PYTHON" -Color Yellow
Write-TimestampedMessage "Setting PYO3_ENVIRONMENT_SIGNATURE to: $env:PYO3_ENVIRONMENT_SIGNATURE" -Color Yellow

# Before CMake configuration, ensure Julia build directory exists and copy artifacts
$juliaBuildDir = Join-Path $buildDir "julia_build"
New-Item -ItemType Directory -Force -Path $juliaBuildDir | Out-Null

# Copy all Julia artifacts
$juliaArtifacts = @(
    "PromptVeilCore.dll",
    "PromptVeilCore.def",
    "PromptVeilCore.exp",
    "PromptVeilCore.lib"
)

foreach ($artifact in $juliaArtifacts) {
    $sourcePath = Join-Path $promptVeilCoreDir $artifact
    $destPath = Join-Path $juliaBuildDir $artifact
    if (Test-Path $sourcePath) {
        Write-TimestampedMessage "Copying Julia artifact: $artifact" -Color Yellow
        Copy-Item -Path $sourcePath -Destination $destPath -Force
        if (-not $?) {
            Write-TimestampedMessage "Warning: Failed to copy $artifact" -Color Yellow
        }
    } else {
        Write-TimestampedMessage "Warning: Julia artifact not found: $artifact" -Color Yellow
    }
}

# Configure and build Rust components via CMake
Write-TimestampedMessage "`nConfiguring CMake..." -Color Yellow
Set-Location build
cmake .. `
    -DBUILD_TESTS=ON `
    -DUSE_GPU=OFF `
    -DBUILD_DOCS=OFF `
    -DCMAKE_BUILD_TYPE=Debug `
    -DPYTHON_EXECUTABLE=$pythonExe

if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: CMake configuration failed" -Color Red
    Set-Location $scriptDir
    exit 1
}

Write-TimestampedMessage "`nBuilding Rust components..." -Color Yellow
cmake --build . --config Debug
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Rust build failed" -Color Red
    Set-Location $scriptDir
    exit 1
}

Set-Location $scriptDir

# 4. Python Package Installation Stage
Write-TimestampedMessage "`n=== Python Package Installation Stage ===" -Color Cyan

Write-TimestampedMessage "`nInstalling Python package..." -Color Yellow

# Install requirements first
Write-TimestampedMessage "Installing requirements from: $scriptDir/promptveil/python/requirements.txt" -Color Yellow
& $pythonExe -m pip install -r "$scriptDir/promptveil/python/requirements.txt"
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Failed to install Python requirements" -Color Red
    exit 1
}

# Set environment variables for the package installation
$env:PROMPTVEIL_JULIA_PATH = Join-Path $buildDir "julia_build"
$env:PROMPTVEIL_RUST_PATH = $buildDir

# Install the package in development mode
Write-TimestampedMessage "`nInstalling PromptVeil package in development mode..." -Color Yellow
Set-Location "$scriptDir/promptveil/python"
& $pythonExe -m pip install -e .
if ($LASTEXITCODE -ne 0) {
    Write-TimestampedMessage "Error: Failed to install PromptVeil package" -Color Red
    Set-Location $scriptDir
    exit 1
}

Set-Location $scriptDir

$endTime = Get-Date
$duration = $endTime - $startTime
Write-TimestampedMessage "`nBuild completed successfully in $($duration.ToString('hh\:mm\:ss\.fff'))!" -Color Green
Write-TimestampedMessage "You can now use the PromptVeil package from Python." -Color Green 