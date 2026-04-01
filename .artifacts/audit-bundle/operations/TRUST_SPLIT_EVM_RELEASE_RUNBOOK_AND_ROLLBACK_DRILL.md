# Trust-Split EVM Release Runbook and Rollback Drill

**Status:** Active (H6)  
**Owner:** Operations + Runtime + Security  
**Purpose:** Standardize release execution and rollback drills with auditable evidence.

---

## 1. Preconditions (Must Pass Before Release)

- `cargo test --lib` passes on release candidate commit.
- Trust policy/convergence tests pass.
- ABI vector and typed provenance tests pass.
- Audit evidence bundle manifest is complete (`bundle.manifest.json`).
- Threat-model traceability matrix is updated for this release.

If any precondition fails, stop and remediate before proceeding.

---

## 2. Release Runbook Checklist

## 2.1 Build and Artifact Freeze

- [ ] Record release tag and commit SHA.
- [ ] Build blockchain artifacts (`.sol`, `.abi`, `.bin`, optional `.http.json`).
- [ ] Generate artifact checksums (SHA-256).
- [ ] Record toolchain versions (`rustc`, `solc`, DAL version).

## 2.2 Policy and Correctness Evidence

- [ ] Export trust policy test logs.
- [ ] Export strict-mode proof logs.
- [ ] Export ABI vector suite logs.
- [ ] Export codegen and provenance shape-lock logs.

## 2.3 Security Evidence

- [ ] Update threat-model traceability matrix with latest run IDs.
- [ ] Export adversarial/security suite summary.
- [ ] Document unresolved findings (or explicit empty report).

## 2.4 Operational Signoff

- [ ] Security reviewer signoff.
- [ ] Runtime owner signoff.
- [ ] Release manager signoff.
- [ ] Bundle manifest finalized and signed (if signature flow enabled).

---

## 3. Rollback Drill Procedure

Perform before production release and record output in artifacts.

## 3.1 Drill Setup

- [ ] Choose staging or forked environment.
- [ ] Deploy candidate artifacts.
- [ ] Capture baseline tx hash/receipt and health checks.

## 3.2 Simulated Incident Trigger

- [ ] Trigger controlled failure scenario (policy mismatch, contract behavior regression, or integration failure).
- [ ] Confirm detection via logs/alerts/checks.

## 3.3 Rollback Execution

- [ ] Execute rollback steps for target environment:
  - revert deployment pointer / registry reference
  - restore previous artifact manifest
  - verify previous version receipts and health
- [ ] Record rollback start/end timestamps and operator.
- [ ] Capture tx hashes and final state verification.

## 3.4 Post-rollback Validation

- [ ] Run smoke tests on rolled-back version.
- [ ] Confirm no unresolved critical errors.
- [ ] Attach rollback evidence logs to audit bundle.

---

## 4. Rollback Drill Record Template

Fill for each release candidate:

- `release_tag`:
- `environment`:
- `incident_scenario`:
- `rollback_operator`:
- `rollback_started_at_utc`:
- `rollback_completed_at_utc`:
- `rollback_tx_hashes`:
- `post_rollback_health_result`:
- `notes`:

---

## 5. Stop Conditions (Fail Closed)

Release is blocked if:

- rollback drill not executed for current release candidate
- rollback drill evidence missing from bundle
- rollback fails to restore previous known-good state
- post-rollback smoke tests fail

