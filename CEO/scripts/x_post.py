#!/usr/bin/env python3
"""
Post tweets to X (Twitter) via API v2. Uses OAuth 1.0a with env vars:
  X_API_KEY, X_API_SECRET, X_ACCESS_TOKEN, X_ACCESS_TOKEN_SECRET

Usage:
  python3 x_post.py "Tweet text here"           # Post new tweet
  python3 x_post.py -                           # Read tweet text from stdin
  python3 x_post.py reply TWEET_ID "Reply text" # Reply to tweet
  python3 x_post.py batch                       # Read JSON from stdin: {"tweets": ["..."], "max_per_batch": 5, "delay_seconds": 2}

Output: JSON to stdout with ok, tweet_id, or error. Batch outputs {ok, posted: [...], failed: [...], total_requested, total_posted}.
"""
import json
import os
import sys
import time

try:
    from requests_oauthlib import OAuth1Session
except ImportError:
    print(json.dumps({"ok": False, "error": "requests-oauthlib not installed. Run: pip install requests-oauthlib"}))
    sys.exit(1)

# Max tweets per batch (safety / rate limit)
MAX_BATCH_SIZE = 10
# Default delay between posts (seconds)
DEFAULT_DELAY_SECONDS = 2


def post_one(oauth, text: str) -> dict:
    """Post a single tweet. Returns {ok, tweet_id} or {ok: False, error}."""
    if not text or not text.strip():
        return {"ok": False, "error": "Empty text"}
    text = text.strip()
    if len(text) > 280:
        text = text[:277] + "..."
    payload = {"text": text}
    url = "https://api.twitter.com/2/tweets"
    resp = oauth.post(url, json=payload)
    if resp.status_code in (200, 201):
        data = resp.json()
        tweet_id = data.get("data", {}).get("id")
        return {"ok": True, "tweet_id": tweet_id, "text": text}
    try:
        err_json = resp.json()
        err_detail = err_json.get("detail", err_json.get("errors", resp.text))
    except Exception:
        err_detail = resp.text
    return {"ok": False, "error": str(err_detail), "text": text}


def run_batch(oauth, data: dict) -> dict:
    tweets = data.get("tweets") or []
    max_per_batch = min(int(data.get("max_per_batch") or MAX_BATCH_SIZE), MAX_BATCH_SIZE)
    delay_seconds = max(0, min(60, float(data.get("delay_seconds") or DEFAULT_DELAY_SECONDS)))
    to_post = tweets[:max_per_batch]
    posted = []
    failed = []
    for i, text in enumerate(to_post):
        result = post_one(oauth, text if isinstance(text, str) else str(text))
        if result.get("ok"):
            posted.append({"text": result.get("text", "")[:80], "tweet_id": result.get("tweet_id")})
        else:
            failed.append({"text": (result.get("text") or "")[:80], "error": result.get("error", "unknown")})
        if i < len(to_post) - 1 and delay_seconds > 0:
            time.sleep(delay_seconds)
    return {
        "ok": len(failed) == 0,
        "posted": posted,
        "failed": failed,
        "total_requested": len(tweets),
        "total_posted": len(posted),
    }


def main():
    api_key = os.environ.get("X_API_KEY")
    api_secret = os.environ.get("X_API_SECRET")
    access_token = os.environ.get("X_ACCESS_TOKEN")
    access_token_secret = os.environ.get("X_ACCESS_TOKEN_SECRET")

    if not all([api_key, api_secret, access_token, access_token_secret]):
        print(json.dumps({"ok": False, "error": "X credentials not set. Set X_API_KEY, X_API_SECRET, X_ACCESS_TOKEN, X_ACCESS_TOKEN_SECRET in .env"}))
        sys.exit(1)

    oauth = OAuth1Session(
        api_key,
        client_secret=api_secret,
        resource_owner_key=access_token,
        resource_owner_secret=access_token_secret,
    )

    if len(sys.argv) < 2:
        print(json.dumps({"ok": False, "error": "Usage: x_post.py <text> | x_post.py - | x_post.py reply <tweet_id> <text> | x_post.py batch"}))
        sys.exit(1)

    if sys.argv[1].lower() == "batch":
        if len(sys.argv) > 2:
            with open(sys.argv[2], "r") as f:
                data = json.load(f)
        else:
            try:
                data = json.load(sys.stdin)
            except Exception as e:
                print(json.dumps({"ok": False, "error": "Invalid JSON: " + str(e)}))
                sys.exit(1)
        out = run_batch(oauth, data)
        print(json.dumps(out))
        sys.exit(0 if out.get("ok") else 1)

    if sys.argv[1] == "-":
        text = sys.stdin.read().strip()
        if len(text) > 280:
            text = text[:277] + "..."
        payload = {"text": text}
    elif sys.argv[1].lower() == "reply":
        if len(sys.argv) < 3:
            print(json.dumps({"ok": False, "error": "Usage: x_post.py reply <tweet_id> [text|-]"}))
            sys.exit(1)
        tweet_id = sys.argv[2]
        text = sys.argv[3] if len(sys.argv) > 3 and sys.argv[3] != "-" else sys.stdin.read().strip()
        payload = {"text": text, "reply": {"in_reply_to_tweet_id": tweet_id}}
    else:
        text = sys.argv[1]
        if len(text) > 280:
            text = text[:277] + "..."
        payload = {"text": text}

    if not text:
        print(json.dumps({"ok": False, "error": "Empty text"}))
        sys.exit(1)

    url = "https://api.twitter.com/2/tweets"
    resp = oauth.post(url, json=payload)

    if resp.status_code in (200, 201):
        data = resp.json()
        tweet_id = data.get("data", {}).get("id")
        print(json.dumps({"ok": True, "tweet_id": tweet_id}))
    else:
        err_body = resp.text
        try:
            err_json = resp.json()
            err_detail = err_json.get("detail", err_json.get("errors", err_body))
        except Exception:
            err_detail = err_body
        print(json.dumps({"ok": False, "error": str(err_detail), "status": resp.status_code}))
        sys.exit(1)


if __name__ == "__main__":
    main()
