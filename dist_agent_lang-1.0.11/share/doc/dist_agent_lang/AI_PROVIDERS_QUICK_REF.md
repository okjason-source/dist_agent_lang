# AI Providers Quick Reference

**All Supported AI Providers for DAL**

---

## Built-In Providers

### OpenAI
```bash
export OPENAI_API_KEY="sk-proj-..."
export OPENAI_MODEL="gpt-4"  # optional
dal ai code "Create a token"
```

### Anthropic
```bash
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"  # optional
dal ai code "Create a token"
```

### Ollama (Local)
```bash
ollama serve
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
export DAL_AI_MODEL="codellama"  # optional
dal ai code "Create a token"
```

---

## Cloud Providers (Easy Setup)

### Cohere
```bash
export DAL_AI_PROVIDER="cohere"
export DAL_AI_ENDPOINT="https://api.cohere.ai/v1/generate"
export DAL_AI_API_KEY="your-cohere-key"
export DAL_AI_MODEL="command"
```

### HuggingFace
```bash
export DAL_AI_PROVIDER="huggingface"
export DAL_AI_ENDPOINT="https://api-inference.huggingface.co/models/codellama/CodeLlama-13b-Instruct-hf"
export DAL_AI_API_KEY="hf_..."
```

### Azure OpenAI
```bash
export DAL_AI_PROVIDER="azure-openai"
export DAL_AI_ENDPOINT="https://YOUR-RESOURCE.openai.azure.com/openai/deployments/YOUR-DEPLOYMENT/chat/completions?api-version=2023-05-15"
export DAL_AI_API_KEY="your-azure-key"
```

### Together AI
```bash
export DAL_AI_PROVIDER="together-ai"
export DAL_AI_ENDPOINT="https://api.together.xyz/v1/chat/completions"
export DAL_AI_API_KEY="your-together-key"
export DAL_AI_MODEL="mistralai/Mixtral-8x7B-Instruct-v0.1"
```

### OpenRouter
```bash
export DAL_AI_PROVIDER="openrouter"
export DAL_AI_ENDPOINT="https://openrouter.ai/api/v1/chat/completions"
export DAL_AI_API_KEY="sk-or-..."
export DAL_AI_MODEL="anthropic/claude-3.5-sonnet"
```

### Replicate
```bash
export DAL_AI_PROVIDER="replicate"
export DAL_AI_ENDPOINT="https://api.replicate.com/v1/predictions"
export DAL_AI_API_KEY="r8_..."
export DAL_AI_MODEL="meta/llama-2-70b-chat"
```

---

## Self-Hosted / Custom

### Any OpenAI-Compatible API
```bash
export DAL_AI_PROVIDER="custom"
export DAL_AI_ENDPOINT="http://your-server:8000/v1/chat/completions"
export DAL_AI_API_KEY="optional"
export DAL_AI_MODEL="your-model"
```

### vLLM
```bash
export DAL_AI_PROVIDER="custom"
export DAL_AI_ENDPOINT="http://localhost:8000/v1/chat/completions"
export DAL_AI_MODEL="meta-llama/Llama-2-13b-chat-hf"
```

### LocalAI
```bash
export DAL_AI_PROVIDER="custom"
export DAL_AI_ENDPOINT="http://localhost:8080/v1/chat/completions"
export DAL_AI_MODEL="ggml-gpt4all-j"
```

### text-generation-webui
```bash
export DAL_AI_PROVIDER="custom"
export DAL_AI_ENDPOINT="http://localhost:5000/v1/chat/completions"
```

---

## Runtime Configuration (DAL Code)

### Built-in Helpers
```dal
// OpenAI
ai.configure_openai("sk-proj-...", "gpt-4")

// Anthropic
ai.configure_anthropic("sk-ant-...", "claude-3-5-sonnet-20241022")

// Cohere
ai.configure_cohere("cohere-key", "command")

// HuggingFace
ai.configure_huggingface("hf_...", "codellama/CodeLlama-13b-Instruct-hf")

// Azure OpenAI
ai.configure_azure_openai("https://...", "azure-key", "gpt-4")

// Together AI
ai.configure_together_ai("together-key", "mixtral-8x7b")

// OpenRouter
ai.configure_openrouter("sk-or-...", "anthropic/claude-3.5-sonnet")

// Replicate
ai.configure_replicate("r8_...", "meta/llama-2-70b-chat")

// Local
ai.configure_local("http://localhost:11434/api/generate", "codellama")

// Any Custom
ai.configure_custom("provider-name", "https://...", "api-key", "model")
```

### Full Configuration
```dal
ai.set_ai_config({
    provider: "provider-name",
    api_key: "your-key",
    endpoint: "https://...",
    model: "model-name",
    temperature: 0.7,
    max_tokens: 2000,
    timeout_seconds: 30
})
```

---

## Config File (.dal/ai_config.toml)

```toml
# Choose one provider
provider = "openai"  # or anthropic, cohere, azure-openai, etc.

# Authentication
api_key = "your-key"

# Endpoint (for custom providers)
endpoint = "https://api.provider.com/..."

# Model
model = "model-name"

# Optional parameters
temperature = 0.7
max_tokens = 2000
timeout_seconds = 30
```

---

## Provider Comparison

| Provider | Speed | Cost | Quality | Privacy | Setup |
|----------|-------|------|---------|---------|-------|
| **OpenAI GPT-4** | ⭐⭐⭐ | 💰💰💰 | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **OpenAI GPT-3.5** | ⭐⭐⭐⭐⭐ | 💰 | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Anthropic Claude** | ⭐⭐⭐⭐ | 💰💰 | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Cohere** | ⭐⭐⭐⭐ | 💰 | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **HuggingFace** | ⭐⭐⭐ | 💰 | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ |
| **Together AI** | ⭐⭐⭐⭐⭐ | 💰 | ⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐ |
| **Ollama** | ⭐⭐⭐⭐ | FREE | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Self-Hosted** | ⭐⭐⭐ | FREE | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |

---

## Cost Estimates (per 1M tokens)

| Provider | Input | Output | Total |
|----------|-------|--------|-------|
| OpenAI GPT-4 | $30 | $60 | ~$45 avg |
| OpenAI GPT-3.5 | $1.5 | $2 | ~$1.75 avg |
| Anthropic Claude Sonnet | $3 | $15 | ~$9 avg |
| Cohere Command | $1 | $2 | ~$1.5 avg |
| Together AI | $0.2 | $0.8 | ~$0.5 avg |
| Ollama | FREE | FREE | FREE |
| Self-Hosted | FREE | FREE | FREE |

---

## Choose By Use Case

### For Production
✅ **OpenAI GPT-4** or **Anthropic Claude**

### For Development
✅ **Ollama** (free) or **Together AI** (cheap)

### For Privacy
✅ **Self-Hosted** or **Ollama**

### For Cost
✅ **Together AI** or **Self-Hosted**

### For Enterprise
✅ **Azure OpenAI** or **Anthropic**

---

## Quick Troubleshooting

### Command to Test
```bash
dal ai code "print hello world"
```

### Check Configuration
```dal
let config = ai.get_ai_config()
print("Provider: " + config.provider)
print("Model: " + config.model)
```

### Debug Mode
```bash
RUST_LOG=debug dal ai code "test"
```

---

## Documentation

- [CUSTOM_AI_PROVIDERS.md](./CUSTOM_AI_PROVIDERS.md) - Complete guide
- [AI_PROVIDER_SETUP.md](./AI_PROVIDER_SETUP.md) - Setup instructions
- [AI_CONFIGURATION_METHODS.md](./AI_CONFIGURATION_METHODS.md) - Configuration methods
