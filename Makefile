# dist_agent_lang Makefile
# Common development tasks and build automation

.PHONY: help build test clean install uninstall fmt clippy bench docs package release

# Default target
help: ## Show this help message
	@echo "dist_agent_lang Development Makefile"
	@echo "==================================="
	@echo ""
	@echo "Available targets:"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-15s %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "Examples:"
	@echo "  make build      # Build the project"
	@echo "  make test       # Run all tests"
	@echo "  make install    # Install dist_agent_lang"
	@echo "  make package    # Create release package"

# Build targets
build: ## Build the project in debug mode
	cargo build

build-release: ## Build the project in release mode
	cargo build --release

# Testing targets
test: ## Run all tests
	cargo test

test-all: ## Run all tests with all features
	cargo test --all-features

test-examples: ## Run example tests
	cargo test --examples

# Code quality targets
fmt: ## Format code with rustfmt
	cargo fmt

clippy: ## Run clippy linter
	cargo clippy

check: ## Check code without building
	cargo check

# Benchmarking
bench: ## Run benchmarks
	cargo bench

# Documentation
docs: ## Generate documentation
	cargo doc --no-deps --open

docs-build: ## Build documentation without opening
	cargo doc --no-deps

# Installation targets
install: build-release ## Install dist_agent_lang
	cargo install --path .

install-local: build-release ## Install to local bin directory
	@mkdir -p ~/.local/bin
	cp target/release/dist_agent_lang ~/.local/bin/
	chmod +x ~/.local/bin/dist_agent_lang
	@echo "dist_agent_lang installed to ~/.local/bin/"
	@echo "Add ~/.local/bin to your PATH if not already there"

uninstall: ## Uninstall dist_agent_lang
	cargo uninstall dist_agent_lang

# Cleaning targets
clean: ## Clean build artifacts
	cargo clean

clean-all: ## Clean everything including target and node_modules
	cargo clean
	rm -rf node_modules
	rm -rf dist_agent_lang-*.tar.gz
	rm -rf dist_agent_lang-*.zip
	rm -rf dist_agent_lang-*

# Packaging targets
package: build-release ## Create release package (minimal - only essential files)
	@echo "Creating minimal release package..."
	@mkdir -p dist_agent_lang-1.0.0/bin
	@cp target/release/dist_agent_lang dist_agent_lang-1.0.0/bin/
	@mkdir -p dist_agent_lang-1.0.0/examples
	@for file in examples/*.dal; do \
		if [ -f "$$file" ] && ! echo "$$file" | grep -q "test_"; then \
			cp "$$file" dist_agent_lang-1.0.0/examples/; \
		fi \
	done
	@cp README.md LICENSE CHANGELOG.md dist_agent_lang-1.0.0/
	@cp scripts/install.sh dist_agent_lang-1.0.0/
	@chmod +x dist_agent_lang-1.0.0/install.sh
	@tar -czf dist_agent_lang-1.0.0.tar.gz dist_agent_lang-1.0.0/
	@zip -r dist_agent_lang-1.0.0.zip dist_agent_lang-1.0.0/
	@echo "Minimal release packages created:"
	@echo "  - dist_agent_lang-1.0.0.tar.gz"
	@echo "  - dist_agent_lang-1.0.0.zip"
	@echo "Contents: binary, examples (.dal files only), README, LICENSE, CHANGELOG, install script"

release: test-all clippy package ## Create release (test, lint, package)
	@echo "Release created successfully!"

# Development targets
dev: ## Start development mode with hot reload
	cargo watch -x check -x test -x run

watch: ## Watch for changes and run tests
	cargo watch -x test

# Example targets
run-examples: ## Run all examples
	@echo "Running examples..."
	@for example in examples/*.rs; do \
		echo "Running $$example..."; \
		cargo run --example $$(basename $$example .rs) || true; \
	done

# Configuration targets
setup: ## Initial setup for development
	@echo "Setting up development environment..."
	rustup update
	cargo install cargo-watch
	cargo install cargo-audit
	@echo "Development environment setup complete!"

# Security targets
audit: ## Run security audit
	cargo audit

# Performance targets
profile: ## Run performance profiling
	cargo build --release
	perf record --call-graph=dwarf target/release/dist_agent_lang
	perf report

# Docker targets
docker-build: ## Build Docker image
	docker build -t dist_agent_lang:latest .

docker-run: ## Run Docker container
	docker run -it dist_agent_lang:latest

# CI/CD targets
ci: test-all clippy audit ## Run CI pipeline
	@echo "CI pipeline completed successfully!"

# Utility targets
version: ## Show version information
	@echo "dist_agent_lang version: 1.0.0"
	@echo "Rust version: $(shell rustc --version)"
	@echo "Cargo version: $(shell cargo --version)"

deps: ## Show dependency tree
	cargo tree

update: ## Update dependencies
	cargo update

# Platform-specific targets
install-linux: ## Install on Linux
	@echo "Installing on Linux..."
	sudo apt-get update
	sudo apt-get install -y build-essential pkg-config libssl-dev
	make install-local

install-macos: ## Install on macOS
	@echo "Installing on macOS..."
	brew install openssl pkg-config
	make install-local

install-windows: ## Install on Windows
	@echo "Installing on Windows..."
	@echo "Please use the Windows installer or run: cargo install --path ."

# Helpers
.PHONY: check-deps
check-deps: ## Check if all dependencies are installed
	@echo "Checking dependencies..."
	@command -v cargo >/dev/null 2>&1 || { echo "Cargo not found. Please install Rust: https://rustup.rs/"; exit 1; }
	@command -v node >/dev/null 2>&1 || { echo "Node.js not found. Please install Node.js: https://nodejs.org/"; exit 1; }
	@echo "All dependencies are installed!"

.PHONY: validate
validate: check-deps fmt clippy test ## Validate code quality
	@echo "Code validation completed successfully!"
