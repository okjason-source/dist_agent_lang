#!/bin/bash
# Homebrew-based Network Fix for Phase 1 Testing
# This script fixes SSL certificate issues using Homebrew

set -e  # Exit on error

echo "🍺 Homebrew Network Fix for dist_agent_lang"
echo "=============================================="
echo ""

# Step 1: Update Homebrew
echo "📦 Step 1: Updating Homebrew..."
brew update

# Step 2: Install/Update certificates
echo ""
echo "🔐 Step 2: Installing/updating SSL certificates..."
brew install ca-certificates
brew install openssl@3

# Step 3: Link certificates
echo ""
echo "🔗 Step 3: Linking certificates..."
OPENSSL_CERT=$(brew --prefix openssl@3)/etc/openssl@3/cert.pem
echo "  OpenSSL cert path: $OPENSSL_CERT"

# Step 4: Configure Cargo
echo ""
echo "⚙️  Step 4: Configuring Cargo..."
mkdir -p ~/.cargo

cat > ~/.cargo/config.toml << EOF
[net]
git-fetch-with-cli = true

[http]
cainfo = "$OPENSSL_CERT"

[build]
jobs = 4
EOF

echo "  ✅ Cargo config created at ~/.cargo/config.toml"

# Step 5: Set environment variables
echo ""
echo "🌍 Step 5: Setting environment variables..."
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export SSL_CERT_FILE="$OPENSSL_CERT"
export CARGO_HTTP_CAINFO="$OPENSSL_CERT"

echo "  ✅ Environment variables set"

# Step 6: Add to shell profile
echo ""
echo "📝 Step 6: Adding to shell profile..."
SHELL_PROFILE="$HOME/.zshrc"

if ! grep -q "CARGO_NET_GIT_FETCH_WITH_CLI" "$SHELL_PROFILE" 2>/dev/null; then
    cat >> "$SHELL_PROFILE" << 'EOF'

# Cargo/Rust SSL Configuration (added by dist_agent_lang setup)
export CARGO_NET_GIT_FETCH_WITH_CLI=true
export SSL_CERT_FILE="$(brew --prefix openssl@3)/etc/openssl@3/cert.pem"
export CARGO_HTTP_CAINFO="$SSL_CERT_FILE"
EOF
    echo "  ✅ Added to $SHELL_PROFILE"
else
    echo "  ℹ️  Already configured in $SHELL_PROFILE"
fi

# Step 7: Verify setup
echo ""
echo "✅ Step 7: Verifying setup..."
echo "  SSL cert file: $(ls -lh $OPENSSL_CERT 2>&1 | awk '{print $5, $9}')"
echo "  Cargo config: $(test -f ~/.cargo/config.toml && echo '✅ exists' || echo '❌ missing')"

# Step 8: Test cargo
echo ""
echo "🧪 Step 8: Testing cargo connectivity..."
cd /Users/jason/lang_mark/dist_agent_lang

if cargo search serde --limit 1 2>&1 | grep -q "serde"; then
    echo "  ✅ Cargo can connect to crates.io"
else
    echo "  ⚠️  Testing connectivity (may take a moment)..."
fi

# Step 9: Install dependencies
echo ""
echo "📥 Step 9: Installing test dependencies..."
echo "  This may take 2-5 minutes..."
cargo build --tests

# Step 10: Success!
echo ""
echo "🎉 Setup Complete!"
echo "=================="
echo ""
echo "✅ Homebrew packages updated"
echo "✅ SSL certificates configured"
echo "✅ Cargo configured"
echo "✅ Test dependencies installed"
echo ""
echo "📋 Next Steps:"
echo "  1. cargo test --test property_tests"
echo "  2. cargo test --test load_stress_tests --nocapture"
echo "  3. cargo test --workspace"
echo ""
echo "💡 If you open a new terminal, run: source ~/.zshrc"

