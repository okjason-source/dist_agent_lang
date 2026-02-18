# AI Configuration Methods - Complete Guide

**All the Ways to Configure AI in DAL**

DAL supports **5 different configuration methods** for AI providers. Choose what works best for your workflow!

---

## Method 1: Environment Variables (Quick & Simple)

**Best for:** Quick setup, CI/CD, temporary configuration

### Usage

```bash
# OpenAI
export OPENAI_API_KEY="sk-proj-..."
export OPENAI_MODEL="gpt-4"  # optional

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"  # optional

# Local (Ollama)
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
export DAL_AI_MODEL="codellama"  # optional

# Advanced settings
export DAL_AI_TEMPERATURE="0.7"  # optional
export DAL_AI_MAX_TOKENS="2000"  # optional
export DAL_AI_TIMEOUT="30"  # optional

# Use DAL
dal ai code "Create a token contract"
```

### Pros
- ✅ Quick to set up
- ✅ Works immediately
- ✅ Easy to override
- ✅ Standard Unix pattern

### Cons
- ❌ Not persistent across sessions (unless added to .zshrc/.bashrc)
- ❌ Environment gets cluttered

---

## Method 2: Config File (.dal/ai_config.toml)

**Best for:** Project-specific settings, team collaboration, persistent configuration

### Setup

Create `.dal/ai_config.toml` in your project:

```toml
# Provider: openai, anthropic, local, or custom
provider = "openai"

# API credentials
api_key = "sk-proj-..."

# Model selection
model = "gpt-4"

# Generation parameters
temperature = 0.7
max_tokens = 2000
timeout_seconds = 30
```

### Examples

**OpenAI Project:**
```toml
# .dal/ai_config.toml
provider = "openai"
openai_model = "gpt-4"
temperature = 0.7
max_tokens = 2000
```

**Anthropic Project:**
```toml
# .dal/ai_config.toml
provider = "anthropic"
anthropic_model = "claude-3-5-sonnet-20241022"
temperature = 0.8
max_tokens = 4000
```

**Local Development:**
```toml
# .dal/ai_config.toml
provider = "local"
endpoint = "http://localhost:11434/api/generate"
model = "codellama"
temperature = 0.7
```

### Multiple Config Locations (Priority Order)

1. `.dal/ai_config.toml` (project-specific)
2. `dal_config.toml` (project root)
3. `.dalconfig` (project root)
4. `~/.dal/config.toml` (user global)

### Usage

```bash
# Just use DAL - it loads config automatically!
dal ai code "Create a REST API"
dal ai explain myfile.dal

# Config file takes precedence
# Override with environment variable if needed
OPENAI_MODEL="gpt-3.5-turbo" dal ai code "test"
```

### Pros
- ✅ Persistent configuration
- ✅ Version control friendly (can commit template)
- ✅ Team can share settings
- ✅ Project-specific
- ✅ Cleaner than environment variables

### Cons
- ❌ API keys in file (add to .gitignore!)
- ❌ Needs file creation

### Security Note

**NEVER commit API keys!**

```bash
# Add to .gitignore
echo ".dal/ai_config.toml" >> .gitignore

# Or use template without keys
cp .dal/ai_config.toml .dal/ai_config.toml.template
# Remove api_key line from template
# Commit template, ignore actual config
```

---

## Method 3: Runtime Configuration (DAL Code)

**Best for:** Dynamic configuration, user preferences, app-specific settings

### Usage in DAL Code

```dal
// Configure OpenAI at runtime
ai.configure_openai("sk-proj-...", "gpt-4")

// Or Anthropic
ai.configure_anthropic("sk-ant-...", "claude-3-5-sonnet-20241022")

// Or local model
ai.configure_local("http://localhost:11434/api/generate", "codellama")

// Now generate text
let code = ai.generate_text("Create a function that adds two numbers")
print(code)
```

### Advanced Configuration

```dal
// Full configuration object
let config = {
    provider: "openai",
    api_key: "sk-proj-...",
    model: "gpt-4",
    temperature: 0.8,
    max_tokens: 3000,
    timeout_seconds: 60
}

ai.set_ai_config(config)

// Use it
let result = ai.generate_text("Complex prompt...")
```

### Get Current Configuration

```dal
// Check what's configured
let config = ai.get_ai_config()
print("Provider: " + config.provider)
print("Model: " + config.model)
```

### Pros
- ✅ Dynamic configuration
- ✅ Can change providers mid-execution
- ✅ User preference handling
- ✅ Conditional logic

### Cons
- ❌ Requires code changes
- ❌ More complex

---

## Method 4: .env File (Modern Approach)

**Best for:** Development, keeping secrets out of version control

### Setup

1. Create `.env` file in project root:

```bash
# .env
OPENAI_API_KEY=sk-proj-...
OPENAI_MODEL=gpt-4
DAL_AI_TEMPERATURE=0.7
DAL_AI_MAX_TOKENS=2000
```

2. Add to `.gitignore`:

```bash
echo ".env" >> .gitignore
```

3. Load automatically (DAL reads from environment):

```bash
# DAL automatically picks up environment variables
dal ai code "Create a token"
```

### Create .env.template for Teams

```bash
# .env.template (commit this)
OPENAI_API_KEY=your-key-here
OPENAI_MODEL=gpt-4
DAL_AI_TEMPERATURE=0.7

# Developers copy and fill in:
# cp .env.template .env
# Then edit .env with real API key
```

### Pros
- ✅ Standard dev practice
- ✅ Easy to share template
- ✅ Works with many tools
- ✅ Cleaner than shell exports

### Cons
- ❌ Requires tool to load .env (or manual export)
- ❌ Still need to gitignore

---

## Method 5: Mixed/Hybrid (Recommended for Production)

**Best for:** Production apps, flexibility, fallback support

### Strategy

```
Priority (highest to lowest):
1. Runtime configuration (ai.configure_*)
2. Environment variables (export VAR=...)
3. Config file (.dal/ai_config.toml)
4. Defaults/Fallback
```

### Example Setup

**Development:**
```toml
# .dal/ai_config.toml
provider = "local"
endpoint = "http://localhost:11434/api/generate"
model = "codellama"
```

**CI/CD:**
```bash
# GitHub Actions / GitLab CI
export OPENAI_API_KEY="${{ secrets.OPENAI_API_KEY }}"
export OPENAI_MODEL="gpt-3.5-turbo"  # Cheaper for tests
```

**Production:**
```bash
# Production environment
export ANTHROPIC_API_KEY="${ANTHROPIC_KEY}"
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
export DAL_AI_MAX_TOKENS="4000"
```

**User Override:**
```bash
# User can always override
OPENAI_MODEL="gpt-4" dal ai code "important task"
```

### Pros
- ✅ Maximum flexibility
- ✅ Different configs for different environments
- ✅ User overrides always work
- ✅ Graceful fallbacks

---

## Comparison Matrix

| Method | Ease of Use | Persistence | Team Friendly | Security | Best For |
|--------|-------------|-------------|---------------|----------|----------|
| **Env Vars** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ | Quick testing |
| **Config File** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | Projects |
| **Runtime** | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | Apps |
| **.env File** | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Development |
| **Hybrid** | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Production |

---

## Real-World Examples

### Example 1: Solo Developer

```bash
# Quick setup with env vars
export OPENAI_API_KEY="sk-..."
dal ai code "Create a DeFi protocol"

# Switch to local for experimentation
export OPENAI_API_KEY=""
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
dal ai code "Experiment with ideas"
```

---

### Example 2: Small Team

**Setup once:**
```bash
# Create config template
cat > .dal/ai_config.toml.template <<EOF
provider = "openai"
# api_key = "YOUR-KEY-HERE"
model = "gpt-4"
temperature = 0.7
EOF

# Commit template
git add .dal/ai_config.toml.template .gitignore
git commit -m "Add AI config template"
```

**Each developer:**
```bash
# Copy and configure
cp .dal/ai_config.toml.template .dal/ai_config.toml
nano .dal/ai_config.toml  # Add your API key

# Use DAL
dal ai code "Team project feature"
```

---

### Example 3: Enterprise/Production

**Project structure:**
```
my-project/
├── .dal/
│   ├── ai_config.toml          # Git-ignored, real keys
│   └── ai_config.toml.template # Committed, no keys
├── .env                         # Git-ignored
├── .env.template                # Committed
└── deploy/
    ├── dev.env                  # Dev environment
    ├── staging.env              # Staging environment
    └── prod.env                 # Production environment
```

**Development:**
```toml
# .dal/ai_config.toml
provider = "local"
endpoint = "http://localhost:11434/api/generate"
```

**Staging:**
```bash
# staging.env
export OPENAI_API_KEY="${STAGING_OPENAI_KEY}"
export OPENAI_MODEL="gpt-3.5-turbo"
export DAL_AI_MAX_TOKENS="1000"
```

**Production:**
```bash
# prod.env (loaded by Kubernetes/Docker)
export ANTHROPIC_API_KEY="${PROD_ANTHROPIC_KEY}"
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"
export DAL_AI_MAX_TOKENS="4000"
export DAL_AI_TIMEOUT="60"
```

---

## Configuration Priority (How It Works)

DAL checks configuration in this order:

```
1. Runtime configuration (ai.configure_*)
   ↓ If not set...
2. Environment variables
   ↓ If not set...
3. Config file (.dal/ai_config.toml)
   ↓ If not found...
4. Defaults (fallback to basic mode)
```

### Example Priority

```bash
# Config file says: "use codellama locally"
# .dal/ai_config.toml
provider = "local"
model = "codellama"

# But environment variable overrides it
export OPENAI_API_KEY="sk-..."

# Result: Uses OpenAI (env var wins)
dal ai code "test"

# Can still force local by unsetting
OPENAI_API_KEY="" dal ai code "test"  # Now uses local
```

---

## Complete Configuration Reference

### Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `OPENAI_API_KEY` | OpenAI API key | `sk-proj-...` |
| `OPENAI_MODEL` | OpenAI model | `gpt-4` |
| `ANTHROPIC_API_KEY` | Anthropic API key | `sk-ant-...` |
| `ANTHROPIC_MODEL` | Anthropic model | `claude-3-5-sonnet-20241022` |
| `DAL_AI_ENDPOINT` | Local model endpoint | `http://localhost:11434/api/generate` |
| `DAL_AI_MODEL` | Local model name | `codellama` |
| `DAL_AI_TEMPERATURE` | Generation temperature (0-1) | `0.7` |
| `DAL_AI_MAX_TOKENS` | Max tokens to generate | `2000` |
| `DAL_AI_TIMEOUT` | Request timeout (seconds) | `30` |

### Config File Options

```toml
# Required
provider = "openai"  # openai, anthropic, local, custom

# Provider-specific (one of these)
api_key = "sk-..."              # For OpenAI/Anthropic
endpoint = "http://localhost"   # For local

# Model selection
model = "gpt-4"                 # Provider-specific model name
openai_model = "gpt-4"          # Alias for model (when provider=openai)
anthropic_model = "claude-3-..."  # Alias for model (when provider=anthropic)
local_model = "codellama"       # Alias for model (when provider=local)

# Generation parameters
temperature = 0.7               # Float: 0-1 (creativity)
max_tokens = 2000               # Integer: tokens to generate
timeout_seconds = 30            # Integer: request timeout
```

### Runtime Configuration (DAL Code)

```dal
// Quick configuration
ai.configure_openai(api_key, model?)
ai.configure_anthropic(api_key, model?)
ai.configure_local(endpoint, model?)

// Full configuration
ai.set_ai_config({
    provider: "openai",
    api_key: "sk-...",
    model: "gpt-4",
    temperature: 0.7,
    max_tokens: 2000,
    timeout_seconds: 30
})

// Get current config
let config = ai.get_ai_config()
```

---

## Quick Start Recipes

### "I just want it to work"
```bash
export OPENAI_API_KEY="sk-proj-..."
dal ai code "hello world"
```

### "I want free/offline"
```bash
ollama serve
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
dal ai code "hello world"
```

### "I want team configuration"
```bash
# Create config
cat > .dal/ai_config.toml <<EOF
provider = "openai"
model = "gpt-4"
EOF

# Share template
git add .dal/ai_config.toml.template
```

### "I want production-ready"
Use **Method 5: Hybrid** with:
- Config file for defaults
- Environment variables for secrets
- Runtime configuration for dynamic changes

---

## Troubleshooting

### "Which configuration is being used?"

Add debug logging:
```dal
let config = ai.get_ai_config()
print("Provider: " + config.provider)
print("Model: " + config.model)
```

Or run with debug:
```bash
RUST_LOG=debug dal ai code "test"
```

### "Config file not loading"

Check these locations in order:
```bash
ls .dal/ai_config.toml        # Project-specific
ls dal_config.toml            # Project root
ls .dalconfig                 # Project root
ls ~/.dal/config.toml         # User global
```

### "Environment variables not working"

Check if set:
```bash
echo $OPENAI_API_KEY
echo $DAL_AI_ENDPOINT
```

Make sure to `export`:
```bash
# Wrong
OPENAI_API_KEY="sk-..."

# Right
export OPENAI_API_KEY="sk-..."
```

---

## Summary

**DAL gives you maximum flexibility:**

- ✅ **5 configuration methods** - Choose what fits your workflow
- ✅ **Priority system** - Runtime > Env > File > Default
- ✅ **Multiple providers** - OpenAI, Anthropic, Local, Custom
- ✅ **Team-friendly** - Share templates, respect overrides
- ✅ **Production-ready** - Secrets management, environment-specific configs

**Recommendation:**
- Development: Config file + local models
- CI/CD: Environment variables
- Production: Hybrid (env vars for secrets, config for settings)
- Solo dev: Whatever's easiest for you!

---

## Next Steps

- See [AI_PROVIDER_SETUP.md](./AI_PROVIDER_SETUP.md) for provider-specific setup
- See [AI_API_INTEGRATION.md](./AI_API_INTEGRATION.md) for technical implementation
- See [CLI_PHASE3_COMPLETE.md](./CLI_PHASE3_COMPLETE.md) for AI command documentation
