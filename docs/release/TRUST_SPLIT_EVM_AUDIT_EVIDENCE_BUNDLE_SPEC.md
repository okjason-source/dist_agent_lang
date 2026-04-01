# Trust-Split EVM Audit Evidence Bundle Spec

**Status:** Active (H6)  
**Owner:** Security + Operations + DevEx  
**Purpose:** Define a deterministic, auditor-facing release bundle that fails closed when required evidence is missing.

---

## 1. Bundle Location and Naming

- Bundle root: `artifacts/audit/<release_tag>/`
- Required manifest filename: `bundle.manifest.json`
- Optional signature sidecar: `bundle.manifest.sig`

The manifest is the source of truth for release evidence completeness.

---

## 2. Required Directory Layout

Each release candidate bundle must include:

- `policy/`
  - trust-mode policy report (`decentralized` deterministic checks, hybrid split report)
  - strict-mode no-silent-fallback logs
- `correctness/`
  - ABI vector test summary and raw logs
  - typed deploy/call provenance contract test logs
- `security/`
  - threat-model-to-test traceability matrix (CSV or Markdown table export)
  - adversarial suite summary and unresolved findings list (explicitly empty if none)
- `deployment/`
  - artifact checksums (`.sol`, `.abi`, `.bin`, optional `.http.json`)
  - deployment metadata (chain IDs, compiler/runtime versions, CI run IDs)
- `operations/`
  - release runbook execution checklist
  - rollback drill execution checklist and outcomes

---

## 3. Manifest Schema (Contract)

`bundle.manifest.json` must contain the following top-level fields:

- `release_tag` (string)
- `git_commit` (string, full SHA)
- `generated_at_utc` (RFC3339 string)
- `ci` (object)
  - `provider` (string)
  - `workflow` (string)
  - `run_id` (string)
- `toolchain` (object)
  - `rust_version` (string)
  - `solc_version` (string)
  - `dal_version` (string)
- `evidence` (object)
  - `policy` (array of file entries)
  - `correctness` (array of file entries)
  - `security` (array of file entries)
  - `deployment` (array of file entries)
  - `operations` (array of file entries)
- `checksums` (array of file checksum entries)
- `signoff` (object)
  - `security_reviewer`
  - `runtime_owner`
  - `release_manager`
  - `approved_at_utc`

### File Entry Shape

Each file entry in `evidence.*` must include:

- `path` (bundle-relative path)
- `description` (short purpose)
- `required` (boolean, must be `true` for all Gate-6 required artifacts)

### Checksum Entry Shape

Each `checksums[]` item must include:

- `path` (bundle-relative path)
- `sha256` (lowercase hex)

---

## 4. Fail-Closed Rules

A release candidate is blocked when any are true:

- a required evidence category is missing
- a required file is missing from disk
- a checksum is missing or does not match
- `signoff` fields are incomplete
- policy logs indicate strict-mode fallback acceptance in production profile
- unresolved high/critical security finding exists without waiver record

---

## 5. Minimum Required Evidence Set (Gate G6)

- **Policy**
  - trust-mode enforcement report
  - strict-mode policy proof logs
- **Correctness**
  - ABI vector suite summary + logs
  - typed provenance shape-lock summary + logs
- **Security**
  - threat-model-to-test matrix (current release)
  - adversarial suite summary + findings
- **Deployment**
  - deterministic artifact checksum list
  - release metadata (versions, commit, chain targets)
- **Operations**
  - runbook completion checklist
  - rollback drill result record

---

## 6. Example Manifest Skeleton

```json
{
  "release_tag": "v0.0.0-rc1",
  "git_commit": "0123456789abcdef0123456789abcdef01234567",
  "generated_at_utc": "2026-03-31T00:00:00Z",
  "ci": {
    "provider": "github-actions",
    "workflow": "ci.yml",
    "run_id": "123456789"
  },
  "toolchain": {
    "rust_version": "rustc 1.0.0",
    "solc_version": "0.8.x",
    "dal_version": "dev"
  },
  "evidence": {
    "policy": [],
    "correctness": [],
    "security": [],
    "deployment": [],
    "operations": []
  },
  "checksums": [],
  "signoff": {
    "security_reviewer": "",
    "runtime_owner": "",
    "release_manager": "",
    "approved_at_utc": ""
  }
}
```

