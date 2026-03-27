# Custom AI Provider Support

**Use ANY AI Provider with DAL - Not Just OpenAI and Anthropic!**

DAL supports **built-in and custom AI providers**, giving you complete flexibility to choose any LLM service or host your own models.

---

## Supported Providers

### ✅ Built-In (Native Support)

| Provider | Type | Configuration | Best For |
|----------|------|---------------|----------|
| **OpenAI** | Cloud | `OPENAI_API_KEY` | Production, GPT-4 |
| **Anthropic** | Cloud | `ANTHROPIC_API_KEY` | Long context, Claude |
| **Ollama** | Local | `DAL_AI_ENDPOINT` | Free, offline, privacy |

### ✅ Custom Providers (Full Support)

| Provider | Type | Configuration | Best For |
|----------|------|---------------|----------|
| **Cohere** | Cloud | Custom config | Multilingual, embeddings |
| **HuggingFace** | Cloud/Self-hosted | Custom config | Open models, flexibility |
| **Azure OpenAI** | Cloud | Custom config | Enterprise, Microsoft |
| **Replicate** | Cloud | Custom config | Easy model hosting |
| **Together AI** | Cloud | Custom config | Fast inference |
| **OpenRouter** | Cloud | Custom config | Multi-model access |
| **Any OpenAI-compatible** | Any | Custom config | Self-hosted, custom |

---

## Quick Start: Custom Providers

### Method 1: Environment Variables

```bash
# Generic custom provider
export DAL_AI_PROVIDER="cohere"
export DAL_AI_ENDPOINT="https://api.cohere.ai/v1/generate"
export DAL_AI_API_KEY="your-cohere-key"
export DAL_AI_MODEL="command"

dal ai code "Create a token contract"
```

### Method 2: Config File

```toml
# .dal/ai_config.toml
provider = "cohere"
endpoint = "https://api.cohere.ai/v1/generate"
api_key = "your-cohere-key"
model = "command"
```

### Method 3: Runtime Configuration

```dal
// In DAL code
ai.configure_custom(
    "cohere",                                    // provider name
    "https://api.cohere.ai/v1/generate",        // endpoint
    "your-cohere-key",                          // API key
    "command"                                   // model (optional)
)

let code = ai.generate_text("Create a function")
```

---

## Provider-Specific Guides

### Cohere

**Setup:**
```bash
# Get API key from https://cohere.com
export DAL_AI_PROVIDER="cohere"
export DAL_AI_ENDPOINT="https://api.cohere.ai/v1/generate"
export DAL_AI_API_KEY="your-cohere-api-key"
export DAL_AI_MODEL="command"  # or "command-light"
```

**Or in code:**
```dal
ai.configure_cohere("your-api-key", "command")
```

**Models:**
- `command` - Most capable
- `command-light` - Faster, cheaper
- `command-nightly` - Latest features

---

### HuggingFace Inference API

**Setup:**
```bash
# Get API key from https://huggingface.co/settings/tokens
export DAL_AI_PROVIDER="huggingface"
export DAL_AI_ENDPOINT="https://api-inference.huggingface.co/models/codellama/CodeLlama-13b-Instruct-hf"
export DAL_AI_API_KEY="hf_..."
export DAL_AI_MODEL="codellama/CodeLlama-13b-Instruct-hf"
```

**Or in code:**
```dal
ai.configure_huggingface(
    "hf_your_token",
    "codellama/CodeLlama-13b-Instruct-hf"
)
```

**Popular Models:**
- `codellama/CodeLlama-13b-Instruct-hf` - Code generation
- `mistralai/Mistral-7B-Instruct-v0.2` - General purpose
- `meta-llama/Llama-2-7b-chat-hf` - Chat
- `bigcode/starcoder` - Code completion

---

### Azure OpenAI

**Setup:**
```bash
# Get from Azure Portal
export DAL_AI_PROVIDER="azure-openai"
export DAL_AI_ENDPOINT="https://your-resource.openai.azure.com/openai/deployments/your-deployment/chat/completions?api-version=2023-05-15"
export DAL_AI_API_KEY="your-azure-key"
export DAL_AI_MODEL="gpt-4"
```

**Or in code:**
```dal
ai.configure_azure_openai(
    "https://your-resource.openai.azure.com/openai/deployments/gpt-4/chat/completions?api-version=2023-05-15",
    "your-azure-key",
    "gpt-4"
)
```

---

### Replicate

**Setup:**
```bash
# Get API key from https://replicate.com
export DAL_AI_PROVIDER="replicate"
export DAL_AI_ENDPOINT="https://api.replicate.com/v1/predictions"
export DAL_AI_API_KEY="r8_..."
export DAL_AI_MODEL="meta/llama-2-70b-chat:latest"
```

**Or in code:**
```dal
ai.configure_replicate(
    "r8_your_token",
    "meta/llama-2-70b-chat:latest"
)
```

**Popular Models:**
- `meta/llama-2-70b-chat` - Most capable Llama
- `mistralai/mixtral-8x7b-instruct-v0.1` - Strong performance
- `google-deepmind/gemma-7b-it` - Google's model

---

### Together AI

**Setup:**
```bash
# Get API key from https://together.ai
export DAL_AI_PROVIDER="together-ai"
export DAL_AI_ENDPOINT="https://api.together.xyz/v1/chat/completions"
export DAL_AI_API_KEY="your-together-key"
export DAL_AI_MODEL="mistralai/Mixtral-8x7B-Instruct-v0.1"
```

**Or in code:**
```dal
ai.configure_together_ai(
    "your-api-key",
    "mistralai/Mixtral-8x7B-Instruct-v0.1"
)
```

**Benefits:**
- Fast inference
- Competitive pricing
- Many models available

---

### OpenRouter

**Setup:**
```bash
# Get API key from https://openrouter.ai
export DAL_AI_PROVIDER="openrouter"
export DAL_AI_ENDPOINT="https://openrouter.ai/api/v1/chat/completions"
export DAL_AI_API_KEY="sk-or-..."
export DAL_AI_MODEL="anthropic/claude-3.5-sonnet"
```

**Or in code:**
```dal
ai.configure_openrouter(
    "sk-or-your-key",
    "anthropic/claude-3.5-sonnet"
)
```

**Benefits:**
- Access to multiple providers through one API
- Compare models easily
- Pay-as-you-go

---

### Self-Hosted / Custom API

**For any OpenAI-compatible API:**

```bash
# vLLM, LocalAI, text-generation-webui, etc.
export DAL_AI_PROVIDER="custom"
export DAL_AI_ENDPOINT="http://your-server:8000/v1/chat/completions"
export DAL_AI_API_KEY="optional-if-auth-required"
export DAL_AI_MODEL="your-model-name"
```

**Or in code:**
```dal
ai.configure_custom(
    "my-custom-api",
    "http://my-server:8000/v1/chat/completions",
    "optional-api-key",
    "llama-2-13b"
)
```

---

## Configuration Methods

### Full Configuration Object

```dal
// Complete control over all settings
ai.set_ai_config({
    provider: "cohere",                           // custom provider name
    api_key: "your-key",                         // authentication
    endpoint: "https://api.cohere.ai/v1/generate", // API endpoint
    model: "command",                            // model name
    temperature: 0.8,                            // creativity (0-1)
    max_tokens: 3000,                            // response length
    timeout_seconds: 60                          // request timeout
})
```

### Environment Variables (Complete List)

```bash
# Provider configuration
export DAL_AI_PROVIDER="provider-name"      # Required for custom
export DAL_AI_ENDPOINT="https://..."        # Required for custom
export DAL_AI_API_KEY="your-key"           # Usually required
export DAL_AI_MODEL="model-name"           # Optional

# Generation parameters
export DAL_AI_TEMPERATURE="0.7"            # Default: 0.7
export DAL_AI_MAX_TOKENS="2000"            # Default: 2000
export DAL_AI_TIMEOUT="30"                 # Default: 30 seconds
```

### Config File (Complete)

```toml
# .dal/ai_config.toml

# Provider settings
provider = "cohere"  # or any provider name
endpoint = "https://api.cohere.ai/v1/generate"
api_key = "your-key"
model = "command"

# Generation parameters
temperature = 0.7
max_tokens = 2000
timeout_seconds = 30
```

---

## Adding New Providers

### Step 1: Identify API Format

DAL auto-detects these formats:
- **OpenAI-compatible** - Most common format
- **Cohere** - Cohere API format
- **HuggingFace** - Inference API format
- **Azure OpenAI** - Azure-specific auth
- **Replicate** - Replicate API format
- **Generic** - Falls back to OpenAI-compatible

### Step 2: Configure Provider

```dal
// If your provider uses OpenAI-compatible format:
ai.configure_custom(
    "my-provider",
    "https://my-api.com/v1/chat/completions",
    "my-api-key",
    "my-model"
)

// DAL will automatically use OpenAI format
```

### Step 3: Test

```bash
dal ai code "hello world"
```

---

## Format Detection

DAL automatically handles different API formats based on provider name:

| Provider Name | Format Used |
|---------------|-------------|
| `cohere` | Cohere API format |
| `huggingface`, `hf` | HuggingFace format |
| `azure`, `azure-openai` | Azure OpenAI format |
| `replicate` | Replicate format |
| `together`, `together-ai` | OpenAI-compatible |
| `openrouter` | OpenAI-compatible |
| **Anything else** | OpenAI-compatible (default) |

**This means most custom providers "just work"** if they use OpenAI-compatible format!

---

## Examples

### Example 1: Try Multiple Providers

```dal
// Try HuggingFace
ai.configure_huggingface("hf_token", "codellama/CodeLlama-13b-Instruct-hf")
let result1 = ai.generate_text("Create a function")

// Try Cohere
ai.configure_cohere("cohere-key", "command")
let result2 = ai.generate_text("Create a function")

// Compare results
print("HuggingFace: " + result1)
print("Cohere: " + result2)
```

### Example 2: Self-Hosted with Fallback

```dal
// Try self-hosted first
ai.configure_custom(
    "local-llm",
    "http://localhost:8080/v1/chat/completions",
    "",
    "llama-2-13b"
)

// If it fails, DAL will fall back to other configured providers
// or basic mode
let code = ai.generate_text("Create API")
```

### Example 3: Cost Optimization

```dal
// Use cheap provider for simple tasks
fn simple_task(prompt: string) -> string {
    ai.configure_cohere("key", "command-light")  // Cheaper
    return ai.generate_text(prompt)
}

// Use expensive provider for complex tasks
fn complex_task(prompt: string) -> string {
    ai.configure_openai("key", "gpt-4")  // More capable
    return ai.generate_text(prompt)
}
```

---

## Pricing Comparison

| Provider | Cost (per 1M tokens) | Speed | Best For |
|----------|---------------------|-------|----------|
| **OpenAI GPT-4** | $30-60 | Medium | Highest quality |
| **OpenAI GPT-3.5** | $1.5-2 | Fast | General use |
| **Anthropic Claude Sonnet** | $3-15 | Medium | Long context |
| **Cohere Command** | $1-2 | Fast | Multilingual |
| **HuggingFace** | $0-2 | Varies | Flexibility |
| **Together AI** | $0.2-1 | Fast | Cost-effective |
| **OpenRouter** | Varies | Fast | Multi-provider |
| **Self-Hosted** | $0 | Varies | Privacy, control |

---

## Common Use Cases

### Use Case 1: Privacy-First (Self-Hosted Only)

```toml
# .dal/ai_config.toml
provider = "custom"
endpoint = "http://internal-server:8080/v1/chat/completions"
model = "llama-2-70b"
# No API key needed for internal service
```

### Use Case 2: Multi-Region (Azure)

```dal
// Use different Azure regions for redundancy
let regions = [
    "https://eastus.openai.azure.com/...",
    "https://westus.openai.azure.com/...",
    "https://europe.openai.azure.com/..."
]

for region in regions {
    ai.configure_azure_openai(region, "key", "gpt-4")
    try {
        return ai.generate_text(prompt)
    } catch {
        continue  // Try next region
    }
}
```

### Use Case 3: Cost Optimization

```dal
// Start with cheap provider
ai.configure_together_ai("key", "mixtral-8x7b")
let result = ai.generate_text(prompt)

// If quality isn't good enough, upgrade
if quality_score(result) < 0.8 {
    ai.configure_openai("key", "gpt-4")
    result = ai.generate_text(prompt)
}
```

---

## Troubleshooting

### "Custom provider failed"

**Check:**
1. Endpoint URL is correct
2. API key is valid
3. Model name is correct
4. Provider name matches supported format

**Debug:**
```bash
RUST_LOG=debug dal ai code "test"
```

### "Invalid response format"

If DAL can't parse the response:

1. Your provider might use a custom format
2. Check the response structure
3. File an issue or implement custom parsing

### "Connection timeout"

```bash
# Increase timeout
export DAL_AI_TIMEOUT="120"  # 2 minutes
```

---

## Advanced: Adding Custom Response Parsers

If your provider uses a completely custom format, you can extend DAL:

```rust
// In ai.rs, add to extract_response_text():
"myprovider" => {
    // Custom parsing logic
    json["my_custom_field"]["nested_text"]
        .as_str()
        .map(|s| s.trim().to_string())
        .ok_or_else(|| "Invalid format".to_string())
}
```

---

## Provider Recommendations

### For Production
1. **OpenAI GPT-4** - Highest quality
2. **Anthropic Claude** - Long context, safety
3. **Azure OpenAI** - Enterprise SLA

### For Development
1. **Ollama (local)** - Free, fast iteration
2. **HuggingFace** - Open models
3. **Together AI** - Good quality/price

### For Privacy
1. **Self-hosted (vLLM, LocalAI)** - Complete control
2. **Ollama** - Easy local setup
3. **On-premise Azure** - Enterprise option

### For Cost
1. **Together AI** - Cheapest cloud
2. **Cohere** - Good value
3. **OpenRouter** - Price comparison

---

## Summary

**DAL supports ANY AI provider:**

✅ **Built-in:** OpenAI, Anthropic, Ollama
✅ **Pre-configured:** Cohere, HuggingFace, Azure, Replicate, Together AI, OpenRouter
✅ **Custom:** Any OpenAI-compatible API
✅ **Self-hosted:** vLLM, LocalAI, text-generation-webui, etc.

**Configuration methods:**
1. Environment variables
2. Config file
3. Runtime configuration
4. .env file
5. Mixed/hybrid

**Choose based on your needs:**
- Quality → OpenAI, Anthropic
- Cost → Together AI, Self-hosted
- Privacy → Self-hosted, Ollama
- Flexibility → HuggingFace, Custom

**It just works!** Most providers use OpenAI-compatible format, so they work automatically.

---

## Related Documentation

- [AI_PROVIDER_SETUP.md](./AI_PROVIDER_SETUP.md) - Quick setup for OpenAI, Anthropic, Ollama
- [AI_CONFIGURATION_METHODS.md](./AI_CONFIGURATION_METHODS.md) - All configuration methods
- [AI_API_INTEGRATION.md](./development/AI_API_INTEGRATION.md) - Technical implementation
- [CLI_PHASE3_COMPLETE.md](./development/CLI_PHASE3_COMPLETE.md) - AI commands documentation
