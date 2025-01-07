# Function to check if a command exists
function Test-Command {
    param($Command)
    try {
        Get-Command $Command -ErrorAction Stop
        return $true
    }
    catch {
        return $false
    }
}

$hasErrors = $false
Write-Host "Checking PromptVeil prerequisites..." -ForegroundColor Cyan

# Check CMake
if (Test-Command "cmake") {
    $version = (cmake --version).Split("`n")[0]
    Write-Host "[OK] CMake found: $version" -ForegroundColor Green
}
else {
    Write-Host "[ERROR] CMake not found. Please install CMake 3.15 or higher." -ForegroundColor Red
    $hasErrors = $true
}

# Check Python
if (Test-Command "python") {
    $version = (python --version)
    Write-Host "[OK] Python found: $version" -ForegroundColor Green
    
    # Check pip (optional now)
    python -m pip --version
    if ($?) {
        Write-Host "[OK] pip found" -ForegroundColor Green
    }
    else {
        Write-Host "[WARNING] pip not found in base Python - will be installed in virtual environment" -ForegroundColor Yellow
    }
}
else {
    Write-Host "[ERROR] Python not found. Please install Python 3.7 or higher." -ForegroundColor Red
    $hasErrors = $true
}

# Check Rust
if (Test-Command "rustc") {
    $version = (rustc --version)
    Write-Host "[OK] Rust found: $version" -ForegroundColor Green
    
    # Check cargo
    if (Test-Command "cargo") {
        $version = (cargo --version)
        Write-Host "[OK] Cargo found: $version" -ForegroundColor Green
    }
    else {
        Write-Host "[ERROR] Cargo not found" -ForegroundColor Red
        $hasErrors = $true
    }
}
else {
    Write-Host "[ERROR] Rust not found. Please install Rust via rustup." -ForegroundColor Red
    $hasErrors = $true
}

# Check Julia
if (Test-Command "julia") {
    $version = (julia --version)
    Write-Host "[OK] Julia found: $version" -ForegroundColor Green
}
else {
    Write-Host "[ERROR] Julia not found. Please install Julia 1.6 or higher." -ForegroundColor Red
    $hasErrors = $true
}

Write-Host "`nChecking Visual Studio Build Tools..." -ForegroundColor Cyan

# Check for Visual Studio or Build Tools
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstallPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsInstallPath) {
        Write-Host "[OK] Visual Studio/Build Tools found at: $vsInstallPath" -ForegroundColor Green
    }
    else {
        Write-Host "[ERROR] Visual C++ Build Tools not found. Please install them through Visual Studio Installer." -ForegroundColor Red
        $hasErrors = $true
    }
}
else {
    Write-Host "[ERROR] Visual Studio Installer not found. Please install Visual Studio or Build Tools." -ForegroundColor Red
    $hasErrors = $true
}

# Exit with error if any checks failed
if ($hasErrors) {
    Write-Host "`nSome prerequisites are missing. Please install them and try again." -ForegroundColor Red
    exit 1
}
else {
    Write-Host "`nAll prerequisites are satisfied!" -ForegroundColor Green
    exit 0
} 