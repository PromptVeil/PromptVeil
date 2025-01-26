# Function to compare version strings
function Compare-Version {
    param(
        [string]$Version1,
        [string]$Version2
    )
    try {
        $v1 = [version]$Version1
        $v2 = [version]$Version2
        return $v1.CompareTo($v2)
    }
    catch {
        Write-Host "Error comparing versions: $Version1 and $Version2" -ForegroundColor Red
        return -1
    }
}

# Function to check if a command exists
function Test-Command {
    param(
        [string]$Command,
        [string]$MinVersion = $null
    )
    try {
        $cmdInfo = Get-Command $Command -ErrorAction Stop
        if ($MinVersion) {
            $version = & $Command --version 2>&1
            Write-Host "[DEBUG] Raw version output for ${Command}: $version" -ForegroundColor Yellow
            
            # Special handling for CMake version
            if ($Command -eq "cmake") {
                Write-Host "[DEBUG] Processing CMake version..." -ForegroundColor Yellow
                $versionStr = $version | Select-Object -First 1  # Get first line only
                Write-Host "[DEBUG] First line: $versionStr" -ForegroundColor Yellow
                
                if ($versionStr -match 'cmake version (\d+\.\d+\.\d+)') {
                    $currentVersion = $matches[1]
                    Write-Host "[DEBUG] Extracted CMake version: $currentVersion" -ForegroundColor Yellow
                    Write-Host "[DEBUG] Comparing with minimum version: $MinVersion" -ForegroundColor Yellow
                    $comparison = Compare-Version $currentVersion $MinVersion
                    Write-Host "[DEBUG] Comparison result: $comparison" -ForegroundColor Yellow
                    
                    if ($comparison -ge 0) {
                        return $true
                    }
                } else {
                    Write-Host "[ERROR] Unable to parse CMake version from: $versionStr" -ForegroundColor Red
                    return $false
                }
            }
            # Default version extraction for other commands
            elseif ($version -match '(\d+\.\d+\.\d+)') {
                $currentVersion = $matches[1]
                if ((Compare-Version $currentVersion $MinVersion) -lt 0) {
                    Write-Host "[ERROR] ${Command} version must be >= $MinVersion (found $currentVersion)" -ForegroundColor Red
                    return $false
                }
            }
            else {
                Write-Host "[ERROR] Unable to parse version for ${Command}" -ForegroundColor Red
                return $false
            }
        }
        return $true
    }
    catch {
        Write-Host "[ERROR] Failed to execute ${Command}: $($_.Exception.Message)" -ForegroundColor Red
        return $false
    }
}

# Array to store missing dependencies
$missingDeps = @()

Write-Host "Checking PromptVeil prerequisites..." -ForegroundColor Cyan

# Check CMake (minimum version 3.15)
if (Test-Command "cmake" "3.15.0") {
    $version = (cmake --version).Split("`n")[0]
    Write-Host "[OK] CMake found: $version" -ForegroundColor Green
}
else {
    Write-Host "[ERROR] CMake not found or version < 3.15.0. Please install CMake 3.15 or higher." -ForegroundColor Red
    $missingDeps += "cmake>=3.15.0"
}

# Check Python (minimum version 3.8)
if (Test-Command "python" "3.8.0") {
    $version = (python --version)
    Write-Host "[OK] Python found: $version" -ForegroundColor Green
    
    # Check pip
    $pipVersion = python -m pip --version 2>&1
    if ($?) {
        Write-Host "[OK] pip found: $pipVersion" -ForegroundColor Green
    }
    else {
        Write-Host "[WARNING] pip not found in base Python - will be installed in virtual environment" -ForegroundColor Yellow
    }
}
else {
    Write-Host "[ERROR] Python not found or version < 3.8.0. Please install Python 3.8 or higher." -ForegroundColor Red
    $missingDeps += "python>=3.8.0"
}

# Check Rust (minimum version 1.70.0)
if (Test-Command "rustc" "1.70.0") {
    $version = (rustc --version)
    Write-Host "[OK] Rust found: $version" -ForegroundColor Green
    
    # Check cargo
    if (Test-Command "cargo") {
        $version = (cargo --version)
        Write-Host "[OK] Cargo found: $version" -ForegroundColor Green
    }
    else {
        Write-Host "[ERROR] Cargo not found" -ForegroundColor Red
        $missingDeps += "cargo"
    }
}
else {
    Write-Host "[ERROR] Rust not found or version < 1.70.0. Please install Rust via rustup." -ForegroundColor Red
    $missingDeps += "rust>=1.70.0"
}

# Check Julia (minimum version 1.6.0)
if (Test-Command "julia" "1.6.0") {
    $version = (julia --version)
    Write-Host "[OK] Julia found: $version" -ForegroundColor Green
}
else {
    Write-Host "[ERROR] Julia not found or version < 1.6.0. Please install Julia 1.6 or higher." -ForegroundColor Red
    $missingDeps += "julia>=1.6.0"
}

Write-Host "`nChecking Visual Studio Build Tools..." -ForegroundColor Cyan

# Check for Visual Studio or Build Tools
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsInstallPath = & $vsWhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if ($vsInstallPath) {
        $vsVersion = & $vsWhere -latest -property catalog_productDisplayVersion
        Write-Host "[OK] Visual Studio/Build Tools found: Version $vsVersion" -ForegroundColor Green
        Write-Host "    Path: $vsInstallPath" -ForegroundColor Green
        
        # Check if MSVC tools are properly installed
        $vcvarsallPath = Join-Path $vsInstallPath "VC\Auxiliary\Build\vcvarsall.bat"
        if (Test-Path $vcvarsallPath) {
            Write-Host "[OK] MSVC tools found" -ForegroundColor Green
        }
        else {
            Write-Host "[ERROR] MSVC tools not found. Please repair Visual Studio installation." -ForegroundColor Red
            $missingDeps += "msvc-tools"
        }
    }
    else {
        Write-Host "[ERROR] Visual C++ Build Tools not found. Please install them through Visual Studio Installer." -ForegroundColor Red
        $missingDeps += "visual-cpp-build-tools"
    }
}
else {
    Write-Host "[ERROR] Visual Studio Installer not found. Please install Visual Studio or Build Tools." -ForegroundColor Red
    $missingDeps += "visual-studio-installer"
}

# Check Windows SDK
$sdkPath = Get-ItemProperty -Path "HKLM:\SOFTWARE\Microsoft\Windows Kits\Installed Roots" -ErrorAction SilentlyContinue
if ($sdkPath -and $sdkPath.KitsRoot10) {
    Write-Host "[OK] Windows SDK found: $($sdkPath.KitsRoot10)" -ForegroundColor Green
}
else {
    Write-Host "[ERROR] Windows SDK not found. Please install Windows SDK." -ForegroundColor Red
    $missingDeps += "windows-sdk"
}

# Save missing dependencies to file
if ($missingDeps.Count -gt 0) {
    $missingDeps | Out-File -FilePath ".missing_deps" -Encoding UTF8
    Write-Host "`nMissing dependencies:" -ForegroundColor Red
    $missingDeps | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
}
else {
    Write-Host "`nAll prerequisites are satisfied!" -ForegroundColor Green
    # Remove missing deps file if it exists
    if (Test-Path ".missing_deps") {
        Remove-Item ".missing_deps"
    }
    exit 0
} 