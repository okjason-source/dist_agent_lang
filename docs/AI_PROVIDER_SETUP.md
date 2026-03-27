# AI Provider Setup Guide

**Choose Your AI Provider - OpenAI, Anthropic, or Local Models**

The `dal ai` commands support **multiple AI providers**. Choose what works best for you:

---

## Option 1: OpenAI (GPT-4, GPT-3.5)

**Best for:** Production use, highest quality responses

### Setup

1. **Get API Key**
   - Go to https://platform.openai.com/api-keys
   - Click "Create new secret key"
   - Copy the key (starts with `sk-proj-` or `sk-`)

2. **Set Environment Variable**
   ```bash
   export OPENAI_API_KEY="sk-proj-..."
   
   # Add to your shell config for persistence
   echo 'export OPENAI_API_KEY="sk-proj-..."' >> ~/.zshrc
   source ~/.zshrc
   ```

3. **Use DAL**
   ```bash
   dal ai code "Create a DeFi lending protocol"
   dal ai explain mycontract.dal
   dal ai audit token.dal
   ```

### Configuration

```bash
# Choose model (default: gpt-4)
export OPENAI_MODEL="gpt-4"          # Most capable
export OPENAI_MODEL="gpt-3.5-turbo"  # Faster, cheaper
```

### Pricing
- GPT-4: ~$0.03/1K input tokens, ~$0.06/1K output tokens
- GPT-3.5: ~$0.0015/1K input tokens, ~$0.002/1K output tokens

---

## Option 2: Anthropic (Claude)

**Best for:** Long context, detailed analysis, code review

### Setup

1. **Get API Key**
   - Go to https://console.anthropic.com/
   - Navigate to "API Keys"
   - Create new key
   - Copy the key (starts with `sk-ant-`)

2. **Set Environment Variable**
   ```bash
   export ANTHROPIC_API_KEY="sk-ant-..."
   
   # Add to your shell config for persistence
   echo 'export ANTHROPIC_API_KEY="sk-ant-..."' >> ~/.zshrc
   source ~/.zshrc
   ```

3. **Use DAL**
   ```bash
   dal ai code "Create a token contract"
   dal ai review complex_system.dal
   dal ai audit defi_protocol.dal
   ```

### Configuration

```bash
# Choose model (default: claude-3-5-sonnet-20241022)
export ANTHROPIC_MODEL="claude-3-5-sonnet-20241022"  # Most capable
export ANTHROPIC_MODEL="claude-3-opus-20240229"     # Highest intelligence
export ANTHROPIC_MODEL="claude-3-haiku-20240307"    # Fastest, cheapest
```

### Pricing
- Claude 3.5 Sonnet: ~$0.003/1K input tokens, ~$0.015/1K output tokens
- Claude 3 Opus: ~$0.015/1K input tokens, ~$0.075/1K output tokens
- Claude 3 Haiku: ~$0.00025/1K input tokens, ~$0.00125/1K output tokens

---

## Option 3: Local Models (Ollama, LM Studio)

**Best for:** Privacy, offline use, free unlimited usage

### Setup with Ollama

1. **Install Ollama**
   ```bash
   # macOS
   brew install ollama
   
   # Linux
   curl -fsSL https://ollama.com/install.sh | sh
   
   # Windows
   # Download from https://ollama.com/download
   ```

2. **Start Ollama Server**
   ```bash
   ollama serve
   ```

3. **Download a Model**
   ```bash
   # Code generation models
   ollama pull codellama         # 7B, good for code
   ollama pull deepseek-coder    # Excellent for coding
   ollama pull phind-codellama   # Optimized for code
   
   # General purpose models
   ollama pull llama2            # Good all-around
   ollama pull mistral           # Fast and capable
   ollama pull llama3            # Latest, most capable
   
   # Small/fast models
   ollama pull tinyllama         # Very fast
   ```

4. **Configure DAL**
   ```bash
   export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
   export DAL_AI_MODEL="codellama"  # or deepseek-coder, llama2, etc.
   
   # Add to shell config
   echo 'export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"' >> ~/.zshrc
   echo 'export DAL_AI_MODEL="codellama"' >> ~/.zshrc
   source ~/.zshrc
   ```

5. **Use DAL (Offline!)**
   ```bash
   dal ai code "Create a REST API"
   dal ai explain myfile.dal
   dal ai test contract.dal
   ```

### Model Recommendations

| Model | Size | Speed | Quality | Best For |
|-------|------|-------|---------|----------|
| `codellama` | 7B | Fast | Good | Code generation |
| `deepseek-coder` | 6.7B | Fast | Excellent | Code, best quality |
| `llama3` | 8B | Medium | Excellent | General purpose |
| `mistral` | 7B | Fast | Good | General purpose |
| `phind-codellama` | 34B | Slow | Excellent | Complex code (needs GPU) |

### Pricing
- **FREE!** Runs locally on your machine
- No API costs
- No rate limits
- Works offline

---

## Using Multiple Providers

You can configure **all three** and DAL will choose automatically:

```bash
# Set all three
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"

# Priority order: OpenAI > Anthropic > Local > Fallback
dal ai code "test"  # Uses OpenAI (first priority)
```

### Override Priority

```bash
# Use Anthropic instead of OpenAI
OPENAI_API_KEY="" dal ai code "test"

# Use local model
OPENAI_API_KEY="" ANTHROPIC_API_KEY="" dal ai code "test"

# Force fallback mode (no AI)
OPENAI_API_KEY="" ANTHROPIC_API_KEY="" DAL_AI_ENDPOINT="" dal ai code "test"
```

---

## Switching Providers

### Team A: OpenAI Only
```bash
# .env file
OPENAI_API_KEY=sk-proj-...
```

### Team B: Anthropic Only
```bash
# .env file
ANTHROPIC_API_KEY=sk-ant-...
```

### Team C: Local Only (Privacy-focused)
```bash
# .env file
DAL_AI_ENDPOINT=http://localhost:11434/api/generate
DAL_AI_MODEL=codellama
```

### Individual Developer: Mix and Match
```bash
# Use OpenAI for code generation (fast)
export OPENAI_API_KEY="sk-..."

# But use local for security audits (private)
dal ai code "Create API"           # Uses OpenAI
OPENAI_API_KEY="" dal ai audit contract.dal  # Uses local (private)
```

---

## Testing Your Setup

### Test OpenAI
```bash
export OPENAI_API_KEY="your-key"
dal ai code "print hello world"
```

### Test Anthropic
```bash
export ANTHROPIC_API_KEY="your-key"
dal ai code "print hello world"
```

### Test Local
```bash
ollama serve
export DAL_AI_ENDPOINT="http://localhost:11434/api/generate"
export DAL_AI_MODEL="codellama"
dal ai code "print hello world"
```

---

## Troubleshooting

### "No API key found"
- Check environment variables: `echo $OPENAI_API_KEY`
- Make sure to `export` the variable
- Try restarting your terminal

### "OpenAI API error 401"
- Invalid API key
- Check key format: should start with `sk-proj-` or `sk-`
- Regenerate key at https://platform.openai.com/api-keys

### "Connection refused" (Local)
- Make sure Ollama is running: `ollama serve`
- Check endpoint: `curl http://localhost:11434/api/generate`
- Try different port if 11434 is blocked

### "Rate limit exceeded"
- OpenAI: Upgrade your account or wait
- Anthropic: Check your usage limits
- Local: No rate limits!

---

## Cost Comparison

### Small Project (100 requests/day)

| Provider | Model | Monthly Cost |
|----------|-------|--------------|
| OpenAI | GPT-3.5 | ~$5 |
| OpenAI | GPT-4 | ~$50 |
| Anthropic | Claude Haiku | ~$2 |
| Anthropic | Claude Sonnet | ~$15 |
| Anthropic | Claude Opus | ~$75 |
| Local | Any | **$0** |

### Medium Project (1000 requests/day)

| Provider | Model | Monthly Cost |
|----------|-------|--------------|
| OpenAI | GPT-3.5 | ~$50 |
| OpenAI | GPT-4 | ~$500 |
| Anthropic | Claude Haiku | ~$20 |
| Anthropic | Claude Sonnet | ~$150 |
| Local | Any | **$0** |

### Recommendation

- **Development/Testing**: Use local models (free)
- **Production/High Quality**: Use OpenAI GPT-4 or Claude Sonnet
- **Budget Conscious**: Use Claude Haiku or GPT-3.5
- **Privacy Sensitive**: Use local models only

---

## Quick Reference

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

# Test
dal ai code "hello world"
```

---

## Security Best Practices

1. **Never commit API keys to git**
   ```bash
   echo ".env" >> .gitignore
   echo "*.key" >> .gitignore
   ```

2. **Use environment variables, not hard-coded keys**
   ```bash
   # Good
   export OPENAI_API_KEY="sk-..."
   
   # Bad - NEVER do this
   # let api_key = "sk-..."
   ```

3. **Rotate keys regularly**
   - Regenerate API keys every 90 days
   - Delete old keys after rotation

4. **Use separate keys for dev/prod**
   ```bash
   # Development
   export OPENAI_API_KEY="sk-dev-..."
   
   # Production
   export OPENAI_API_KEY="sk-prod-..."
   ```

5. **Monitor usage**
   - Check OpenAI usage: https://platform.openai.com/usage
   - Check Anthropic usage: https://console.anthropic.com/

---

## Support

**Need Help?**
- Full integration guide: `docs/development/AI_API_INTEGRATION.md`
- CLI reference: `docs/CLI_QUICK_REFERENCE.md`
- Phase 3 documentation: `docs/development/CLI_PHASE3_COMPLETE.md`

**Can't get API keys?**
- Use local models (Ollama) - completely free and offline
- All commands work in fallback mode (basic templates)

**Want to contribute?**
- Add support for more providers (Cohere, HuggingFace, etc.)
- Improve prompts for better code generation
- Add caching to reduce costs
