# dist_agent_lang Makefile
# Common development tasks and build automation

# Version from Cargo.toml (single source of truth)
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')

.PHONY: help build test clean install install-dal-sync uninstall fmt clippy bench docs docs-bundle package release

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

# One command: refresh target/release/dal and ~/.cargo/bin/dal (optional /usr/local: make install-dal-sync USR_LOCAL=1)
install-dal-sync: ## Build release dal + cargo install --bin dal; add USR_LOCAL=1 for /usr/local/bin link
	@if [ "$(USR_LOCAL)" = "1" ]; then SYNC_DAL_USR_LOCAL=1 ./scripts/sync-all-dal.sh; else ./scripts/sync-all-dal.sh; fi

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

# Documentation bundle (per docs/RELEASE_DOCS_BUNDLE.md)
# Essential: 8 files. Recommended: +10 files.
DOCS_ROOT = docs/PUBLIC_DOCUMENTATION_INDEX.md docs/syntax.md docs/attributes.md docs/CLI_QUICK_REFERENCE.md docs/STDLIB_REFERENCE.md docs/Documentation.md docs/MOLD_FORMAT.md docs/AGENT_CAPABILITIES.md docs/AI_PROVIDERS_QUICK_REF.md docs/VENV.md
DOCS_GUIDES = docs/guides/QUICK_START.md docs/guides/AGENT_SETUP_AND_USAGE.md docs/guides/SKILLS_AND_REGISTRY.md docs/guides/PERSISTENT_AGENT_MEMORY.md docs/guides/API_REFERENCE.md docs/guides/AI_FEATURES_GUIDE.md
DOCS_GETTING_STARTED = docs/getting_started/INSTALLATION.md docs/getting_started/USAGE.md

docs-bundle: ## Create docs-only tarball for release (Essential + Recommended)
	@echo "Creating docs bundle for $(VERSION)..."
	@mkdir -p dist_agent_lang-$(VERSION)-docs/share/doc/dist_agent_lang/guides
	@mkdir -p dist_agent_lang-$(VERSION)-docs/share/doc/dist_agent_lang/getting_started
	@for f in $(DOCS_ROOT); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)-docs/share/doc/dist_agent_lang/; done
	@for f in $(DOCS_GUIDES); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)-docs/share/doc/dist_agent_lang/guides/; done
	@for f in $(DOCS_GETTING_STARTED); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)-docs/share/doc/dist_agent_lang/getting_started/; done
	@tar -czf dist_agent_lang-$(VERSION)-docs.tar.gz dist_agent_lang-$(VERSION)-docs/
	@rm -rf dist_agent_lang-$(VERSION)-docs
	@echo "Docs bundle created: dist_agent_lang-$(VERSION)-docs.tar.gz"

# Packaging targets
package: build-release ## Create release package (binary + examples + docs in share/doc)
	@echo "Creating release package for $(VERSION)..."
	@mkdir -p dist_agent_lang-$(VERSION)/bin
	@mkdir -p dist_agent_lang-$(VERSION)/share/doc/dist_agent_lang/guides
	@mkdir -p dist_agent_lang-$(VERSION)/share/doc/dist_agent_lang/getting_started
	@cp target/release/dal dist_agent_lang-$(VERSION)/bin/
	@chmod +x dist_agent_lang-$(VERSION)/bin/dal
	@mkdir -p dist_agent_lang-$(VERSION)/examples
	@for file in examples/*.dal; do \
		if [ -f "$$file" ] && ! echo "$$file" | grep -q "test_"; then \
			cp "$$file" dist_agent_lang-$(VERSION)/examples/; \
		fi \
	done
	@cp README.md LICENSE CHANGELOG.md dist_agent_lang-$(VERSION)/
	@cp scripts/install.sh dist_agent_lang-$(VERSION)/
	@chmod +x dist_agent_lang-$(VERSION)/install.sh
	@for f in $(DOCS_ROOT); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)/share/doc/dist_agent_lang/; done
	@for f in $(DOCS_GUIDES); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)/share/doc/dist_agent_lang/guides/; done
	@for f in $(DOCS_GETTING_STARTED); do [ -f "$$f" ] && cp "$$f" dist_agent_lang-$(VERSION)/share/doc/dist_agent_lang/getting_started/; done
	@tar -czf dist_agent_lang-$(VERSION).tar.gz dist_agent_lang-$(VERSION)/
	@zip -r dist_agent_lang-$(VERSION).zip dist_agent_lang-$(VERSION)/
	@echo "Release packages created:"
	@echo "  - dist_agent_lang-$(VERSION).tar.gz"
	@echo "  - dist_agent_lang-$(VERSION).zip"
	@echo "Contents: dal binary, examples, docs in share/doc/, README, LICENSE, CHANGELOG, install script"

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
	@echo "dist_agent_lang version: $(VERSION)"
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
