# Handling credentials (email, ordering platforms, APIs)

How to handle credentials (email, ordering platforms, API keys) in dist_agent_lang without putting secrets in source code.

---

## 1. **Prefer environment variables**

- **Do not** hardcode passwords, API keys, or tokens in `.dal` or config files that are committed.
- **Do** use environment variables and inject them at run time (local shell, CI, or deployment).

### Suggested env names (convention)

| Use case | Example variable | Notes |
|----------|------------------|--------|
| Email (SMTP) | `EMAIL_SMTP_HOST`, `EMAIL_SMTP_USER`, `EMAIL_SMTP_PASSWORD`, `EMAIL_FROM` | One set per provider |
| Email (API, e.g. SendGrid) | `SENDGRID_API_KEY` | Or `POSTMARK_API_KEY`, etc. |
| Ordering / commerce | `ORDERING_PLATFORM_API_KEY`, `ORDERING_PLATFORM_SECRET` | Or per platform: `STRIPE_SECRET_KEY`, `SHOPIFY_*`, etc. |
| LLM | `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `DAL_AI_ENDPOINT` | Already used by the runtime for `ai::` |
| DB | `DB_HOST`, `DB_USER`, `DB_PASSWORD`, `DB_NAME` | See CONFIGURATION_GUIDE.md |
| Master key for encrypted secrets | `MASTER_ENCRYPTION_KEY` | 32+ chars if you use config encryption |

- Keep one logical integration per prefix (e.g. one set of `EMAIL_*` for your primary provider, or separate `SENDGRID_*` / `POSTMARK_*` if you use both).

---

## 2. **What’s wired for development**

- **Rust / process:** The runtime and stdlib **already read env** where they’re used. For development, set the vars (or use a `.env` and load it before running `dal`). No extra wiring needed for:
  - **AI:** `OPENAI_API_KEY`, `ANTHROPIC_API_KEY`, `DAL_AI_ENDPOINT`, `OPENAI_BASE_URL`, `OPENAI_MODEL`, etc.
  - **Auth:** `JWT_SECRET`, `AUTH_RATE_LIMIT_*`, `AUTH_ROLE_*`, `AUTH_ROLES_JSON`, etc.
  - **Trust / admin:** `ADMIN_IDS`, `ADMIN_LEVEL_<id>`, etc.
  - **Key store:** `DAL_KEY_STORE`, `DAL_KEY_STORE_PATH`, `DAL_KEY_STRICT`
  - **Transactions:** `DAL_TX_STORAGE`, `DAL_TX_STORAGE_PATH`, `DAL_TX_*`
  - **Mold / chain:** `DAL_PRIVATE_KEY`, `DAL_MOLD_REGISTRY_ADDRESS`, `DAL_RPC_URL`, `DAL_IPFS_API`
  - **IoT:** `IOT_CLOUD_URL`, `IOT_CLOUD_KEY`, `IOT_*_API_URL` / `*_API_KEY`
  - **Log:** `LOG_FILE`, `LOG_DIR`, `LOG_LEVEL`, `LOG_SINK`, etc.
  - **Environment name:** `DIST_AGENT_ENV` (defaults to `development`).
- **CLI:** `dal config show` prints the current env; `dal config get <key>` prints one value. Useful for checking that vars are set.
- **DAL:** The **`config::` namespace is exposed to the DAL runtime.** From a `.dal` file you can call:
  - `config::get_env(key)` – returns the value or errors if missing
  - `config::get_required_env(key)` – same, required
  - `config::get_env_or_default(key, default)` – returns value or default if missing
  - `config::get_database_config()`, `config::get_api_config()`, `config::get_blockchain_config()`, `config::get_ai_config()` – return maps of config (each requires the relevant env vars or they error)

So for **custom credentials** (e.g. email, ordering): set env vars (e.g. `EMAIL_SMTP_USER`, `ORDERING_PLATFORM_API_KEY`) and read them from DAL with `config::get_env("EMAIL_SMTP_USER")` or `config::get_env_or_default("EMAIL_SMTP_USER", "")`.

---

## 3. **Local development: `.env` and `.gitignore`**

- Use a `.env` file in the project root for local dev only.
- **Add `.env` to `.gitignore`** so it is never committed.
- Load `.env` before running (e.g. `source .env` or `export $(cat .env | xargs)`, or a tool like `dotenv`).
- In `.env`, use the same variable names as above, e.g.:

```bash
# .env (do not commit)
EMAIL_SMTP_HOST=smtp.example.com
EMAIL_SMTP_USER=agent@example.com
EMAIL_SMTP_PASSWORD=secret
ORDERING_PLATFORM_API_KEY=sk_...
OPENAI_API_KEY=sk-...
```

---

## 4. **Encrypted secrets (production)**

For production, the codebase supports:

- **`config::` (Rust):** `ConfigManager` with `get_required_env`, `get_env_or_default`, and **encrypted secrets** via `store_secret` / `get_secret` using a master key (see `src/stdlib/config.rs` and CONFIGURATION_GUIDE.md). Use a **strong `MASTER_ENCRYPTION_KEY`** (32+ chars) and never commit it.
- **DAL pattern:** `examples/secure_configuration_example.dal` shows a **SecretsManagementService** with:
  - A vault (e.g. map) keyed by secret name.
  - Access controlled by `cloudadmin::authorize(admin_id, "read"|"write", "secrets")`.
  - Store/retrieve/rotate by name; in production you’d feed real secrets from env or a vault into this service (e.g. at init or via a secure admin path), not from DAL source.

So for **email** and **ordering platforms** in production:

1. Prefer **env vars** in the process (or your deployment secrets manager that injects env).
2. If you need to cache or rotate secrets inside the app, use the **config** encrypted store (Rust) or a **secure service** in DAL that holds them in memory and only exposes them to authorized callers.

---

## 5. **Per-integration layout (email / ordering)**

- **One set of env vars per integration** keeps things clear:
  - Email: `EMAIL_*` or `SENDGRID_*` / `POSTMARK_*`.
  - Ordering: `STRIPE_*`, `SHOPIFY_*`, or a generic `ORDERING_*` if you abstract one platform.
- In DAL, keep a small “credentials/config” layer that:
  - Is initialized at startup (or by a trusted admin) with values that came from env or a secrets service.
  - Exposes only “what’s needed” to the rest of the app (e.g. “send email” or “create order” using already-configured credentials), not the raw secret.

---

## 6. **Checklist**

- [ ] No credentials in source or in committed config.
- [ ] `.env` (if used) is in `.gitignore`.
- [ ] Env vars use a consistent naming convention (e.g. `EMAIL_*`, `ORDERING_*_API_KEY`).
- [ ] Production uses env or an encrypted/vault-backed path; master encryption key is not in repo.
- [ ] Access to secrets in DAL is gated (e.g. `cloudadmin::authorize` or equivalent) where the pattern is used.

For full config and validation details, see **CONFIGURATION_GUIDE.md**. For a DAL pattern with a secrets vault and cloudadmin, see **examples/secure_configuration_example.dal**.
