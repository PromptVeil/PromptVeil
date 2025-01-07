#!/bin/bash

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

# Function to check if we're running on Ubuntu/Debian
is_debian_based() {
    [ -f "/etc/debian_version" ]
}

# Function to check if we're running on RHEL/CentOS/Fedora
is_rhel_based() {
    [ -f "/etc/redhat-release" ]
}

# Function to install packages on Ubuntu/Debian
install_debian_packages() {
    write_timestamped_message "Installing packages on Debian-based system..." "cyan"
    
    # Update package lists
    sudo apt-get update
    
    # Install required packages
    sudo apt-get install -y \
        build-essential \
        cmake \
        pkg-config \
        python3.10-dev \
        python3.10-venv \
        python3-pip \
        curl \
        wget \
        git \
        clang \
        lld \
        libssl-dev \
        libffi-dev \
        zlib1g-dev \
        libbz2-dev \
        libreadline-dev \
        libsqlite3-dev \
        libncurses5-dev \
        libncursesw5-dev \
        xz-utils \
        tk-dev
}

# Function to install packages on RHEL/CentOS/Fedora
install_rhel_packages() {
    write_timestamped_message "Installing packages on RHEL-based system..." "cyan"
    
    # Install EPEL repository if not present
    sudo dnf install -y epel-release
    
    # Install required packages
    sudo dnf groupinstall -y "Development Tools"
    sudo dnf install -y \
        cmake \
        python3.10-devel \
        python3.10-pip \
        python3.10-virtualenv \
        curl \
        wget \
        git \
        clang \
        lld \
        openssl-devel \
        libffi-devel \
        zlib-devel \
        bzip2-devel \
        readline-devel \
        sqlite-devel \
        ncurses-devel \
        xz-devel \
        tk-devel
}

# Function to install Rust
install_rust() {
    write_timestamped_message "Installing Rust..." "cyan"
    if ! command -v rustc &> /dev/null; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
    else
        write_timestamped_message "Rust is already installed" "green"
    fi
}

# Function to install Julia
install_julia() {
    write_timestamped_message "Installing Julia..." "cyan"
    if ! command -v julia &> /dev/null; then
        # Download and extract Julia
        JULIA_VERSION="1.11.2"
        JULIA_MAJOR="1.11"
        JULIA_INSTALL_DIR="/opt/julia-${JULIA_VERSION}"
        
        # Determine system architecture
        ARCH=$(uname -m)
        case "$ARCH" in
            "x86_64")
                JULIA_ARCH="x64"
                ;;
            "aarch64")
                JULIA_ARCH="aarch64"
                ;;
            *)
                write_timestamped_message "Unsupported architecture: $ARCH" "red"
                exit 1
                ;;
        esac
        
        JULIA_URL="https://julialang-s3.julialang.org/bin/linux/${ARCH}/${JULIA_MAJOR}/julia-${JULIA_VERSION}-linux-${JULIA_ARCH}.tar.gz"
        
        write_timestamped_message "Downloading Julia from: $JULIA_URL" "yellow"
        wget -O julia.tar.gz "$JULIA_URL"
        
        write_timestamped_message "Extracting Julia to /opt..." "yellow"
        sudo tar xzf julia.tar.gz -C /opt/
        
        # Verify installation directory
        if [ ! -d "$JULIA_INSTALL_DIR" ]; then
            write_timestamped_message "Error: Julia installation directory not found at $JULIA_INSTALL_DIR" "red"
            exit 1
        fi
        
        # Set correct permissions
        write_timestamped_message "Setting permissions for Julia installation..." "yellow"
        sudo chmod -R 755 "$JULIA_INSTALL_DIR"
        sudo chown -R root:root "$JULIA_INSTALL_DIR"
        
        # Create symbolic link
        write_timestamped_message "Creating symbolic link for Julia..." "yellow"
        sudo ln -sf "$JULIA_INSTALL_DIR/bin/julia" /usr/local/bin/julia
        
        # Verify lib directory
        if [ ! -d "$JULIA_INSTALL_DIR/lib" ]; then
            write_timestamped_message "Error: Julia lib directory not found at $JULIA_INSTALL_DIR/lib" "red"
            exit 1
        fi
        
        # Add Julia lib path to ld.so.conf.d
        write_timestamped_message "Configuring Julia library path..." "yellow"
        echo "$JULIA_INSTALL_DIR/lib" | sudo tee /etc/ld.so.conf.d/julia.conf
        sudo ldconfig
        
        # Clean up
        rm julia.tar.gz
        
        write_timestamped_message "Julia installation completed successfully!" "green"
        write_timestamped_message "Julia binary: $(which julia)" "green"
        write_timestamped_message "Julia version: $(julia --version)" "green"
        write_timestamped_message "Julia lib path: $JULIA_INSTALL_DIR/lib" "green"
    else
        write_timestamped_message "Julia is already installed" "green"
        # Verify existing installation
        JULIA_VERSION="1.11.2"
        JULIA_INSTALL_DIR="/opt/julia-${JULIA_VERSION}"
        
        if [ ! -d "$JULIA_INSTALL_DIR/lib" ]; then
            write_timestamped_message "Warning: Julia lib directory not found at expected location" "red"
            write_timestamped_message "Attempting to fix installation..." "yellow"
            install_julia
        else
            write_timestamped_message "Julia installation verified at: $JULIA_INSTALL_DIR" "green"
            # Ensure library path is configured
            if [ ! -f "/etc/ld.so.conf.d/julia.conf" ]; then
                write_timestamped_message "Configuring Julia library path..." "yellow"
                echo "$JULIA_INSTALL_DIR/lib" | sudo tee /etc/ld.so.conf.d/julia.conf
                sudo ldconfig
            fi
        fi
    fi
}

# Main installation process
write_timestamped_message "Starting dependency installation..." "cyan"

# Check if we have a list of missing dependencies
DEPS_FILE=".missing_deps"
if [ -f "$DEPS_FILE" ]; then
    write_timestamped_message "Found list of missing dependencies" "yellow"
    MISSING_DEPS=($(cat "$DEPS_FILE"))
    write_timestamped_message "Dependencies to install: ${MISSING_DEPS[*]}" "yellow"
else
    write_timestamped_message "No dependency list found, will install all required packages" "yellow"
    MISSING_DEPS=("all")
fi

# Install system packages based on distribution
if is_debian_based; then
    # Only install if we need all deps or specific ones are missing
    if [[ " ${MISSING_DEPS[*]} " =~ "all" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "cmake" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "gcc" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "python" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "make" ]]; then
        install_debian_packages
    fi
elif is_rhel_based; then
    # Only install if we need all deps or specific ones are missing
    if [[ " ${MISSING_DEPS[*]} " =~ "all" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "cmake" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "gcc" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "python" ]] || \
       [[ " ${MISSING_DEPS[*]} " =~ "make" ]]; then
        install_rhel_packages
    fi
else
    write_timestamped_message "Unsupported Linux distribution" "red"
    exit 1
fi

# Install Rust if needed
if [[ " ${MISSING_DEPS[*]} " =~ "all" ]] || [[ " ${MISSING_DEPS[*]} " =~ "rust" ]]; then
    install_rust
fi

# Install Julia if needed
if [[ " ${MISSING_DEPS[*]} " =~ "all" ]] || [[ " ${MISSING_DEPS[*]} " =~ "julia" ]]; then
    install_julia
fi

# Clean up deps file
rm -f "$DEPS_FILE"

write_timestamped_message "All dependencies installed successfully!" "green" 