# Trust-Split EVM Threat Model to Test Traceability Matrix

**Status:** Active (H6)  
**Owner:** Security + QA + Runtime  
**Purpose:** Map each high-priority risk to concrete automated tests and the latest passing evidence.

---

## 1. Usage Rules

- Every release candidate must update this matrix.
- Every `Risk ID` must map to at least one automated test.
- A row is incomplete if `Test ID`, `Last passing run`, or `Evidence artifact` is missing.
- High/Critical risks without passing tests are release blockers unless explicitly waived.

---

## 2. Traceability Matrix

| Risk ID | Threat Scenario | Severity | Control Objective | Test ID(s) | Suite | Last passing run | Evidence artifact |
|---|---|---|---|---|---|---|---|
| TM-001 | Decentralized service executes non-deterministic namespace call (`ai::`, `sh::`, `fs::`, `web::`, etc.) | Critical | Reject at parse/compile with explicit diagnostic | `compile::h5_policy_convergence_tests::parser_and_compiler_agree_on_fs_namespace_rejection` | L0 / Unit | _(fill per release)_ | `correctness/trust-policy-tests.log` |
| TM-002 | `@trust` malformed or missing model silently accepted | High | Fail closed on invalid/missing trust model | `compile::h5_policy_convergence_tests::parser_rejects_trust_without_string_model`<br>`compile::h5_policy_convergence_tests::parser_rejects_invalid_trust_model` | L0 / Unit | _(fill per release)_ | `correctness/trust-policy-tests.log` |
| TM-003 | Policy divergence between parser/compiler allows inconsistent enforcement | High | Keep parser and compiler trust checks aligned | `compile::h5_policy_convergence_tests::forbidden_namespace_lists_are_aligned` | L0 / Unit | _(fill per release)_ | `correctness/trust-policy-tests.log` |
| TM-004 | Decentralized codegen falls back to revert stubs for supported subset | Critical | Emit real Solidity bodies for supported constructs | `compile::blockchain::tests::decentralized_v1_e2e_source_parse_compile_produces_real_solidity` | L1 / Unit | _(fill per release)_ | `correctness/blockchain-codegen-tests.log` |
| TM-005 | Map index access/assignment lowers incorrectly (`self.map[key]`) | High | Correct deterministic lowering and diagnostics | `compile::blockchain::tests::decentralized_v1_lowers_map_index_read_and_write_for_self_fields`<br>`compile::blockchain::tests::decentralized_v1_rejects_non_self_index_access` | L1 / Unit | _(fill per release)_ | `correctness/blockchain-codegen-tests.log` |
| TM-006 | Strict mode silently fabricates deploy/call success | Critical | Strict policy returns explicit machine-readable errors | `stdlib::chain::abi_golden_vectors::typed_call_strict_missing_calldata_uses_missing_required_field_code`<br>`stdlib::chain::abi_golden_vectors::typed_deploy_strict_missing_payload_uses_missing_required_field_code` | L2 / Unit | _(fill per release)_ | `policy/strict-mode-proof.log` |
| TM-007 | ABI decode/revert handling drifts from canonical vectors | High | Keep selector/encode/decode/revert parity locked | `stdlib::chain::abi_golden_vectors::*` (suite run) | L2 / Unit | _(fill per release)_ | `correctness/abi-vectors.log` |
| TM-008 | Overloaded function decode ambiguity causes wrong typed decode | High | Require explicit signature hint on ambiguity | `stdlib::add_sol::tests::registry_requires_function_signature_for_overloaded_name` | L2 / Unit | _(fill per release)_ | `correctness/abi-registry-tests.log` |
| TM-009 | Unknown trust model accidentally receives elevated runtime privileges | High | Runtime fail-closed for unknown models | `compile::h5_policy_convergence_tests::all_three_trust_models_parse_and_compile_without_error` + runtime policy smoke tests | L0/L1 | _(fill per release)_ | `policy/runtime-trust-policy.log` |
| TM-010 | Deployment provenance evidence contract drifts (missing keys) | High | Shape-lock typed evidence maps and audit projections | `stdlib::chain::abi_golden_vectors::*provenance*` (suite run) | L2 / Unit | _(fill per release)_ | `correctness/provenance-shape-lock.log` |

---

## 3. Release Signoff Rules

- Security reviewer confirms every Critical/High row has passing evidence.
- QA confirms `Last passing run` points to the current release candidate run.
- Release manager confirms artifacts exist in the audit bundle and match checksums.

---

## 4. Waiver Record (If Needed)

Use only for time-limited exceptions:

- `Risk ID`
- `Reason`
- `Compensating controls`
- `Expiry date`
- `Approver`

If any waiver expires before release, release is blocked.

