#!/bin/bash
# Quick smoke-test: send a tiny completion request to OpenAI and print the result.
cd "$(dirname "$0")"

if [ -f .env ]; then
  while IFS= read -r line; do
    [[ "$line" =~ ^[[:space:]]*# ]] && continue
    [[ -z "${line// }" ]] && continue
    export "$line"
  done < .env
fi

if [ -z "$OPENAI_API_KEY" ]; then
  echo "OPENAI_API_KEY is not set. Add it to .env first."
  exit 1
fi

MODEL="${OPENAI_MODEL:-gpt-4o}"
echo "Testing key against model: $MODEL ..."

RESPONSE=$(curl -s -w "\n%{http_code}" \
  https://api.openai.com/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -d "{\"model\":\"$MODEL\",\"messages\":[{\"role\":\"user\",\"content\":\"Reply with OK\"}],\"max_tokens\":5}")

HTTP_CODE=$(echo "$RESPONSE" | tail -1)
BODY=$(echo "$RESPONSE" | sed '$d')

if [ "$HTTP_CODE" = "200" ]; then
  echo "HTTP 200 — Key is valid."
  echo "$BODY" | python3 -c "import sys,json; print(json.load(sys.stdin)['choices'][0]['message']['content'])" 2>/dev/null || echo "$BODY"
else
  echo "HTTP $HTTP_CODE — Key may be invalid or account has no credits."
  echo "$BODY"
  exit 1
fi
