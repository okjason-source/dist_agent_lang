# 🚀 Production Deployment Guide (v1.0.1)

> **📢 Beta Release v1.0.1:** For production deployments, conduct thorough testing and consider third-party security audits for critical applications. **Beta testing contributions welcome!** 🙏

Complete guide to deploying **dist_agent_lang** contracts to production networks.

---

## 📋 Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Environment Setup](#environment-setup)
3. [Deployment Methods](#deployment-methods)
4. [Multi-Chain Deployment](#multi-chain-deployment)
5. [Docker Deployment](#docker-deployment)
6. [CI/CD Integration](#cicd-integration)
7. [Monitoring & Maintenance](#monitoring--maintenance)
8. [Security Considerations](#security-considerations)
9. [Troubleshooting](#troubleshooting)

---

## ✅ Pre-Deployment Checklist

Before deploying to production:

### 1. Code Quality
- [ ] All tests passing (`dal test --all`)
- [ ] Linter checks pass (`dal lint --strict`)
- [ ] Code formatted (`dal format .`)
- [ ] Security audit complete (see [Security Guide](SECURITY_GUIDE.md))
- [ ] Performance benchmarks acceptable

### 2. Security
- [ ] Re-entrancy protection enabled
- [ ] Safe math operations verified
- [ ] Access control properly configured
- [ ] Rate limiting configured
- [ ] Input validation comprehensive
- [ ] Oracle data validation enabled

### 3. Configuration
- [ ] Environment variables set
- [ ] Network RPC endpoints configured
- [ ] Gas settings optimized
- [ ] Wallet/keys secured (hardware wallet recommended)
- [ ] Backup wallet configured

### 4. Testing
- [ ] Unit tests: 100% pass
- [ ] Integration tests: 100% pass
- [ ] Testnet deployment successful
- [ ] Load testing complete
- [ ] Security testing complete

### 5. Documentation
- [ ] README updated
- [ ] API documentation generated
- [ ] Deployment notes prepared
- [ ] Rollback plan documented

---

## ⚙️ Environment Setup

### 1. Production Environment Variables

Create a `.env.production` file:

```bash
# Network Configuration (Multi-Chain)
# Note: DAL_NETWORK controls ALL blockchain connections (Ethereum, Polygon, Solana, etc.)
DAL_NETWORK=mainnet  # Use production mainnets for all chains
DAL_RPC_URL=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
DAL_CHAIN_ID=1  # Default: Ethereum mainnet

# Multi-Chain RPC Overrides (Optional)
DAL_ETHEREUM_RPC=https://mainnet.infura.io/v3/YOUR_PROJECT_ID
DAL_POLYGON_RPC=https://polygon-mainnet.infura.io/v3/YOUR_PROJECT_ID
DAL_SOLANA_RPC=https://api.mainnet-beta.solana.com
DAL_ARBITRUM_RPC=https://arb1.arbitrum.io/rpc

# Wallet Configuration
DAL_WALLET_TYPE=hardware  # or 'encrypted', 'keystore'
DAL_WALLET_PATH=/path/to/wallet

# Security Features (NEW in v1.0.2)
DAL_ENABLE_REENTRANCY_GUARD=true
DAL_ENABLE_SAFE_MATH=true
DAL_ENABLE_RATE_LIMITING=true
DAL_JWT_SECRET=your-strong-jwt-secret-32-chars-min  # For API authentication
DAL_JWT_EXPIRATION_HOURS=24
DAL_ENABLE_REPLAY_PROTECTION=true  # Nonce-based signature validation
DAL_ENABLE_STRUCTURED_LOGGING=true  # Enhanced security audit logs

# CloudAdmin Configuration (NEW in v1.0.2)
DAL_CLOUDADMIN_ENABLED=true
DAL_SUPERADMIN_ADDRESSES=0xYourAddress1,0xYourAddress2  # Comma-separated
DAL_ADMIN_POLICY_LEVEL=strict  # strict, moderate, or permissive
DAL_HYBRID_TRUST_ENABLED=true  # Enable hybrid trust model

# Gas Configuration
DAL_GAS_PRICE=auto  # or specific gwei value
DAL_GAS_LIMIT=auto
DAL_MAX_PRIORITY_FEE=2  # gwei

# Oracle Configuration
DAL_ORACLE_PROVIDER=chainlink
DAL_ORACLE_TIMEOUT=30000  # ms
DAL_ORACLE_REQUIRE_SIGNATURE=true
DAL_ORACLE_MIN_CONFIRMATIONS=3  # Multi-source validation

# Deployment
DAL_VERIFY_ON_ETHERSCAN=true
DAL_ETHERSCAN_API_KEY=YOUR_API_KEY
DAL_VERIFY_ON_POLYGONSCAN=true
DAL_POLYGONSCAN_API_KEY=YOUR_API_KEY

# Monitoring & Logging
DAL_ENABLE_TELEMETRY=true
DAL_SENTRY_DSN=https://your-sentry-dsn
DAL_LOG_LEVEL=info  # debug, info, warn, error
DAL_SECURITY_LOG_TARGET=security::auth  # Structured logging target
DAL_AUDIT_LOG_PATH=/var/log/dal/audit.log
```

### 2. Secure Your Private Keys

**Option A: Hardware Wallet (Most Secure)**
```bash
# Configure Ledger
dal wallet setup --type ledger --device /dev/hidraw0

# Configure Trezor
dal wallet setup --type trezor
```

**Option B: Encrypted Keystore**
```bash
# Create encrypted keystore
dal wallet create --type keystore --password-file password.txt

# Store keystore securely
chmod 600 keystore.json
```

**Option C: Environment Variables (Development Only)**
```bash
# NOT recommended for production
export DAL_PRIVATE_KEY="0x..."
```

---

## 🌍 Deployment Methods

### Method 1: CLI Deployment (Recommended)

```bash
# Deploy single contract
dal deploy contracts/MyContract.dal \
  --network mainnet \
  --env production \
  --verify \
  --constructor arg1 arg2 arg3

# Deploy multiple contracts
dal deploy contracts/ \
  --network mainnet \
  --env production \
  --sequence deployment-order.yaml
```

**deployment-order.yaml:**
```yaml
contracts:
  - name: Token
    file: contracts/Token.dal
    constructor:
      - "MyToken"
      - "MTK"
      - 1000000
    
  - name: Marketplace
    file: contracts/Marketplace.dal
    constructor:
      - "$Token"  # Reference to previously deployed Token
    depends_on:
      - Token
```

### Method 2: Deployment Script

Create `deploy.sh`:

```bash
#!/bin/bash
set -e

echo "🚀 Starting production deployment..."

# Load environment
source .env.production

# Compile contracts
echo "📦 Compiling contracts..."
dal compile contracts/ --optimize --release

# Run tests one more time
echo "🧪 Running final tests..."
dal test --all --network mainnet-fork

# Deploy
echo "🌐 Deploying to mainnet..."
dal deploy contracts/ \
  --network mainnet \
  --verify \
  --sequence deployment-order.yaml \
  --dry-run

# Confirm deployment
read -p "Continue with actual deployment? (yes/no) " -r
if [[ $REPLY == "yes" ]]; then
  dal deploy contracts/ \
    --network mainnet \
    --verify \
    --sequence deployment-order.yaml
  
  echo "✅ Deployment complete!"
  echo "📝 Saving deployment info..."
  dal deployment save deployments/mainnet-$(date +%Y%m%d).json
else
  echo "❌ Deployment cancelled"
  exit 1
fi
```

### Method 3: Programmatic Deployment

Create `deploy.ts`:

```typescript
import { Deployer, Network } from 'dist-agent-lang';

async function main() {
  // Initialize deployer
  const deployer = new Deployer({
    network: Network.Mainnet,
    signer: await getHardwareWallet(),
    verify: true,
  });
  
  // Deploy Token
  console.log('Deploying Token...');
  const token = await deployer.deploy('contracts/Token.dal', {
    constructor: ['MyToken', 'MTK', 1000000],
    gasLimit: 5000000,
  });
  
  console.log(`Token deployed at: ${token.address}`);
  
  // Deploy Marketplace
  console.log('Deploying Marketplace...');
  const marketplace = await deployer.deploy('contracts/Marketplace.dal', {
    constructor: [token.address],
    gasLimit: 8000000,
  });
  
  console.log(`Marketplace deployed at: ${marketplace.address}`);
  
  // Verify contracts
  await deployer.verify(token);
  await deployer.verify(marketplace);
  
  // Save deployment info
  await deployer.saveDeployment('deployments/mainnet.json');
  
  console.log('✅ Deployment complete!');
}

main().catch(console.error);
```

---

## 🌐 Multi-Chain Deployment

### Deploy to Multiple Chains Simultaneously

```bash
# Deploy to Ethereum, Polygon, and Arbitrum
dal deploy contracts/MyContract.dal \
  --networks ethereum,polygon,arbitrum \
  --env production \
  --verify-all
```

### Chain-Specific Configuration

Create `chains.yaml`:

```yaml
ethereum:
  rpc: https://mainnet.infura.io/v3/PROJECT_ID
  chain_id: 1
  gas_price: auto
  verify_api: https://api.etherscan.io/api
  verify_key: $ETHERSCAN_API_KEY

polygon:
  rpc: https://polygon-mainnet.infura.io/v3/PROJECT_ID
  chain_id: 137
  gas_price: 50  # gwei
  verify_api: https://api.polygonscan.com/api
  verify_key: $POLYGONSCAN_API_KEY

arbitrum:
  rpc: https://arb1.arbitrum.io/rpc
  chain_id: 42161
  gas_price: auto
  verify_api: https://api.arbiscan.io/api
  verify_key: $ARBISCAN_API_KEY
```

Deploy with config:

```bash
dal deploy contracts/ \
  --chains-config chains.yaml \
  --networks ethereum,polygon,arbitrum
```

---

## 🐳 Docker Deployment

### Dockerfile

Create `Dockerfile`:

```dockerfile
FROM rust:1.75 as builder

# Install dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source
WORKDIR /app
COPY . .

# Build
RUN cargo build --release

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/dal /usr/local/bin/

# Copy contracts
COPY contracts /app/contracts
COPY .env.production /app/.env

WORKDIR /app

# Expose ports (if running HTTP server)
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s \
  CMD dal status || exit 1

# Run
CMD ["dal", "serve", "--config", "/app/.env"]
```

### Build and Run

```bash
# Build image
docker build -t my-dal-app:latest .

# Run container
docker run -d \
  --name my-dal-app \
  -p 8080:8080 \
  -v $(pwd)/.env.production:/app/.env \
  -v $(pwd)/keystore:/app/keystore:ro \
  --restart unless-stopped \
  my-dal-app:latest
```

### Docker Compose

Create `docker-compose.yml`:

```yaml
version: '3.8'

services:
  dal-app:
    build: .
    image: my-dal-app:latest
    container_name: dal-app
    ports:
      - "8080:8080"
    environment:
      - DAL_NETWORK=mainnet
      - DAL_LOG_LEVEL=info
    volumes:
      - ./contracts:/app/contracts:ro
      - ./keystore:/app/keystore:ro
      - ./logs:/app/logs
      - ./.env.production:/app/.env:ro
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "dal", "status"]
      interval: 30s
      timeout: 10s
      retries: 3
    networks:
      - dal-network

  # Optional: Add monitoring
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml:ro
    networks:
      - dal-network

  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    networks:
      - dal-network

networks:
  dal-network:
    driver: bridge
```

---

## 🔄 CI/CD Integration

### GitHub Actions

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Production

on:
  push:
    tags:
      - 'v*.*.*'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build
        run: cargo build --release
      
      - name: Run tests
        run: cargo test --all
      
      - name: Lint
        run: |
          ./target/release/dal lint --strict contracts/
      
      - name: Security check
        run: cargo audit

  deploy:
    needs: test
    runs-on: ubuntu-latest
    environment: production
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup environment
        run: |
          echo "${{ secrets.ENV_PRODUCTION }}" > .env.production
      
      - name: Deploy to Ethereum
        run: |
          ./target/release/dal deploy contracts/ \
            --network mainnet \
            --env production \
            --verify
        env:
          DAL_PRIVATE_KEY: ${{ secrets.DEPLOYER_PRIVATE_KEY }}
          DAL_INFURA_KEY: ${{ secrets.INFURA_PROJECT_ID }}
          DAL_ETHERSCAN_KEY: ${{ secrets.ETHERSCAN_API_KEY }}
      
      - name: Save deployment info
        uses: actions/upload-artifact@v3
        with:
          name: deployment-info
          path: deployments/
      
      - name: Notify success
        if: success()
        run: |
          curl -X POST ${{ secrets.SLACK_WEBHOOK }} \
            -H 'Content-Type: application/json' \
            -d '{"text":"✅ Production deployment successful!"}'
      
      - name: Notify failure
        if: failure()
        run: |
          curl -X POST ${{ secrets.SLACK_WEBHOOK }} \
            -H 'Content-Type: application/json' \
            -d '{"text":"❌ Production deployment failed!"}'
```

### GitLab CI

Create `.gitlab-ci.yml`:

```yaml
stages:
  - build
  - test
  - deploy

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo

cache:
  paths:
    - .cargo/
    - target/

build:
  stage: build
  image: rust:1.75
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/dal

test:
  stage: test
  image: rust:1.75
  script:
    - cargo test --all
    - ./target/release/dal lint --strict contracts/

deploy_production:
  stage: deploy
  image: rust:1.75
  only:
    - tags
  environment:
    name: production
  script:
    - echo "$ENV_PRODUCTION" > .env.production
    - ./target/release/dal deploy contracts/ --network mainnet --verify
  when: manual
```

---

## 📊 Monitoring & Maintenance

### Health Checks

```bash
# Check deployment status
dal status --network mainnet --contract MyContract

# Monitor gas usage
dal monitor gas --network mainnet --contract MyContract

# Check transaction status
dal tx status 0x1234567890abcdef...
```

### Logging

Configure structured logging:

```bash
# Start with logging
dal serve --log-file /var/log/dal/app.log --log-level info

# Rotate logs
dal logs rotate --max-size 100M --max-age 30d
```

### Metrics & Alerts

Setup Prometheus metrics:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'dal-app'
    static_configs:
      - targets: ['dal-app:8080']
```

---

## 🔒 Security Considerations

### 1. Private Key Management
- ✅ Use hardware wallets for production
- ✅ Encrypt keystores with strong passwords
- ✅ Use environment-specific keys
- ❌ Never commit private keys
- ❌ Never log private keys

### 2. Deployment Security
- ✅ Use `--dry-run` before actual deployment
- ✅ Verify contract source code on block explorers
- ✅ Enable all security features (@reentrancy_guard, @safe_math)
- ✅ Set up monitoring and alerts
- ✅ Have a pause mechanism for emergencies

### 3. Post-Deployment
- ✅ Renounce ownership if appropriate
- ✅ Set up multi-sig for critical operations
- ✅ Monitor for unusual activity
- ✅ Keep emergency contacts ready

---

## 🆘 Troubleshooting

### Deployment Fails

**Problem**: "Insufficient funds"
```bash
# Check balance
dal wallet balance --network mainnet

# Get estimated cost
dal estimate contracts/MyContract.dal --network mainnet
```

**Problem**: "Nonce too low"
```bash
# Reset nonce
dal wallet reset-nonce --network mainnet
```

**Problem**: "Gas estimation failed"
```bash
# Specify gas manually
dal deploy contracts/MyContract.dal --gas-limit 5000000
```

### Verification Fails

```bash
# Retry verification
dal verify 0xContractAddress --network mainnet

# Verify with constructor args
dal verify 0xContractAddress \
  --constructor-args arg1 arg2 arg3 \
  --network mainnet
```

---

## 📚 Additional Resources

- [Security Guide](SECURITY_GUIDE.md) - Comprehensive security best practices
- [Performance Guide](PERFORMANCE_GUIDE.md) - Optimize gas and execution
- [Best Practices](BEST_PRACTICES.md) - Code organization and patterns
- [API Reference](API_REFERENCE.md) - Complete stdlib documentation

---

**Next:** [Best Practices Guide →](BEST_PRACTICES.md)

