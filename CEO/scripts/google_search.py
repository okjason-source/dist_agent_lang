#!/usr/bin/env python3
"""Google Custom Search for agent_assistant. Prints a short summary for the LLM.
Usage: python3 scripts/google_search.py 'your query'
Requires: GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX in .env (or environment)."""
import os
import sys
import urllib.parse
import urllib.request
import json

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 scripts/google_search.py 'query'")
        sys.exit(1)
    query = " ".join(sys.argv[1:]).strip()
    if not query:
        print("No query provided.")
        sys.exit(1)

    key = os.environ.get("GOOGLE_SEARCH_API_KEY") or os.environ.get("GOOGLE_API_KEY")
    cx = os.environ.get("GOOGLE_SEARCH_CX") or os.environ.get("GOOGLE_CSE_ID")
    if not key or not cx:
        print("Google search not configured. Set GOOGLE_SEARCH_API_KEY and GOOGLE_SEARCH_CX in .env (use the search action as fallback).")
        sys.exit(1)

    url = "https://www.googleapis.com/customsearch/v1?key={}&cx={}&q={}".format(
        key, cx, urllib.parse.quote_plus(query)
    )
    try:
        req = urllib.request.Request(url)
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read().decode())
    except Exception as e:
        print("Search failed: " + str(e))
        sys.exit(1)

    items = data.get("items") or []
    if not items:
        print("No results for: " + query)
        return

    lines = []
    for i, item in enumerate(items[:5], 1):
        title = item.get("title", "")
        snippet = item.get("snippet", "")
        link = item.get("link", "")
        lines.append("{}. {} - {}\n   {}".format(i, title, link, snippet))
    print("\n".join(lines))

if __name__ == "__main__":
    main()
