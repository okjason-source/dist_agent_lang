#!/bin/bash
# Verify that required env vars are set before starting the agent.
cd "$(dirname "$0")"

if [ -f .env ]; then
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// }" ]] && continue
    export "$line"
  done < .env
fi

echo "=== Agent Assistant Environment Check ==="
echo ""

if [ -n "$OPENAI_API_KEY" ]; then
  echo "OPENAI_API_KEY     : SET (length ${#OPENAI_API_KEY})"
else
  echo "OPENAI_API_KEY     : NOT set"
fi

if [ -n "$ANTHROPIC_API_KEY" ]; then
  echo "ANTHROPIC_API_KEY  : SET (length ${#ANTHROPIC_API_KEY})"
else
  echo "ANTHROPIC_API_KEY  : not set (optional)"
fi

echo "OPENAI_MODEL       : ${OPENAI_MODEL:-not set (uses default)}"
echo "DAL_OPENAI_API_KEY : ${DAL_OPENAI_API_KEY:+SET}${DAL_OPENAI_API_KEY:-not set (auto-derived)}"
echo ""

if [ -z "$OPENAI_API_KEY" ] && [ -z "$ANTHROPIC_API_KEY" ]; then
  echo "WARNING: No AI API key found. Copy .env.example to .env and add your key."
  exit 1
fi

echo "Environment OK."
