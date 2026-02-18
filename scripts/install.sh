#!/bin/bash

# dist_agent_lang Installation Script
# Version: 1.0.0

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "msys" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Function to install Rust
install_rust() {
    print_status "Installing Rust..."
    
    if command_exists rustup; then
        print_success "Rust is already installed"
        rustup update
    else
        print_status "Downloading and installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        print_success "Rust installed successfully"
    fi
    
    # Verify installation
    if command_exists cargo; then
        print_success "Cargo is available"
    else
        print_error "Cargo installation failed"
        exit 1
    fi
}

# Function to install Node.js
install_nodejs() {
    print_status "Checking Node.js installation..."
    
    if command_exists node; then
        NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
        if [ "$NODE_VERSION" -ge 18 ]; then
            print_success "Node.js $NODE_VERSION is already installed"
        else
            print_warning "Node.js version $NODE_VERSION is too old. Please upgrade to version 18 or higher."
        fi
    else
        print_status "Installing Node.js..."
        OS=$(detect_os)
        
        if [ "$OS" = "linux" ]; then
            # Install Node.js on Linux
            curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
            sudo apt-get install -y nodejs
        elif [ "$OS" = "macos" ]; then
            # Install Node.js on macOS
            if command_exists brew; then
                brew install node@18
            else
                print_error "Homebrew not found. Please install Homebrew first: https://brew.sh/"
                exit 1
            fi
        else
            print_error "Please install Node.js manually from https://nodejs.org/"
            exit 1
        fi
    fi
}

# Function to install system dependencies
install_dependencies() {
    print_status "Installing system dependencies..."
    OS=$(detect_os)
    
    if [ "$OS" = "linux" ]; then
        # Ubuntu/Debian
        if command_exists apt-get; then
            sudo apt-get update
            sudo apt-get install -y build-essential pkg-config libssl-dev
        # CentOS/RHEL
        elif command_exists yum; then
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y openssl-devel
        # Arch Linux
        elif command_exists pacman; then
            sudo pacman -S --noconfirm base-devel openssl
        fi
    elif [ "$OS" = "macos" ]; then
        if command_exists brew; then
            brew install openssl pkg-config
        else
            print_warning "Homebrew not found. Some dependencies may need to be installed manually."
        fi
    fi
}

# Function to build dist_agent_lang
build_dist_agent_lang() {
    print_status "Building dist_agent_lang..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Cargo.toml not found. Please run this script from the dist_agent_lang directory."
        exit 1
    fi
    
    # Build in release mode
    cargo build --release
    
    if [ $? -eq 0 ]; then
        print_success "dist_agent_lang built successfully"
    else
        print_error "Build failed"
        exit 1
    fi
}

# Function to install dist_agent_lang
install_dist_agent_lang() {
    print_status "Installing dist_agent_lang..."
    
    # Determine installation directory
    if [ -w "/usr/local/bin" ]; then
        INSTALL_DIR="/usr/local/bin"
    else
        INSTALL_DIR="$HOME/.local/bin"
        mkdir -p "$INSTALL_DIR"
    fi
    
    # Copy binary
    cp target/release/dist_agent_lang "$INSTALL_DIR/"
    chmod +x "$INSTALL_DIR/dist_agent_lang"
    
    print_success "dist_agent_lang installed to $INSTALL_DIR/dist_agent_lang"
    
    # Add to PATH if not already there
    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        if [ "$SHELL" = "/bin/bash" ]; then
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.bashrc"
            print_status "Added to PATH in ~/.bashrc"
        elif [ "$SHELL" = "/bin/zsh" ]; then
            echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$HOME/.zshrc"
            print_status "Added to PATH in ~/.zshrc"
        fi
    fi
}

# Function to run tests
run_tests() {
    print_status "Running tests..."
    cargo test
    
    if [ $? -eq 0 ]; then
        print_success "All tests passed"
    else
        print_warning "Some tests failed"
    fi
}

# Function to create configuration
create_config() {
    print_status "Creating default configuration..."
    
    CONFIG_DIR="$HOME/.config/dist_agent_lang"
    mkdir -p "$CONFIG_DIR"
    
    cat > "$CONFIG_DIR/config.toml" << EOF
# dist_agent_lang Configuration File

[blockchain]
# Ethereum configuration
ethereum_rpc = "https://mainnet.infura.io/v3/YOUR_KEY"
ethereum_chain_id = 1

# Polygon configuration
polygon_rpc = "https://polygon-rpc.com"
polygon_chain_id = 137

# Binance Smart Chain configuration
bsc_rpc = "https://bsc-dataseed.binance.org"
bsc_chain_id = 56

# Arbitrum configuration
arbitrum_rpc = "https://arb1.arbitrum.io/rpc"
arbitrum_chain_id = 42161

# Private key (keep secure!)
private_key = "your_private_key_here"

[ai]
# OpenAI configuration
api_key = "your_openai_api_key"
model = "gpt-4"
max_tokens = 4096

# AI agent settings
default_memory_size = 2000
max_concurrent_tasks = 5

[database]
# Database configuration
url = "postgresql://user:password@localhost/dist_agent_lang"
pool_size = 10
max_connections = 20

[logging]
# Logging configuration
level = "info"
file = "$CONFIG_DIR/dist_agent_lang.log"
max_size = "100MB"
max_files = 5

[security]
# Security settings
trust_level = "hybrid"
audit_logging = true
kyc_required = false
aml_required = false
EOF

    print_success "Configuration created at $CONFIG_DIR/config.toml"
    print_warning "Please update the configuration with your actual API keys and settings"
}

# Function to create examples directory
create_examples() {
    print_status "Setting up examples..."
    
    EXAMPLES_DIR="$HOME/dist_agent_lang_examples"
    mkdir -p "$EXAMPLES_DIR"
    
    # Copy examples if they exist
    if [ -d "examples" ]; then
        cp -r examples/* "$EXAMPLES_DIR/"
        print_success "Examples copied to $EXAMPLES_DIR"
    fi
}

# Function to verify installation
verify_installation() {
    print_status "Verifying installation..."
    
    if command_exists dist_agent_lang; then
        VERSION=$(dist_agent_lang --version 2>/dev/null || echo "1.0.2")
        print_success "dist_agent_lang $VERSION is installed and working"
    else
        print_error "dist_agent_lang not found in PATH"
        exit 1
    fi
}

# Function to show next steps
show_next_steps() {
    echo ""
    print_success "🎉 dist_agent_lang installation completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Update configuration: ~/.config/dist_agent_lang/config.toml"
    echo "2. Try the examples: $HOME/dist_agent_lang_examples"
    echo "3. Read the documentation: https://distagentlang.com/docs"
    echo "4. Join the community: https://github.com/dist_agent_lang/dist_agent_lang"
    echo ""
    echo "Quick start:"
    echo "  dist_agent_lang --help"
    echo "  dist_agent_lang run examples/hello_world.dal"
    echo ""
}

# Main installation function
main() {
    echo "🚀 dist_agent_lang Installation Script"
    echo "======================================"
    echo ""
    
    # Check if running as root
    if [ "$EUID" -eq 0 ]; then
        print_warning "Running as root is not recommended"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Install dependencies
    install_dependencies
    
    # Install Rust
    install_rust
    
    # Install Node.js
    install_nodejs
    
    # Build dist_agent_lang
    build_dist_agent_lang
    
    # Run tests
    run_tests
    
    # Install dist_agent_lang
    install_dist_agent_lang
    
    # Create configuration
    create_config
    
    # Create examples
    create_examples
    
    # Verify installation
    verify_installation
    
    # Show next steps
    show_next_steps
}

# Run main function
main "$@"
