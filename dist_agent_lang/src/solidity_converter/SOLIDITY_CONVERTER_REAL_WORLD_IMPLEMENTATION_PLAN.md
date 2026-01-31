# Solidity Converter Real-World Implementation Plan

This document catalogs comments in the solidity converter that indicate placeholders, simplified logic, or future work, and proposes a prioritized plan for real-world implementations.

**Scope:** `src/solidity_converter/` (parser.rs, analyzer.rs, generator.rs, converter.rs, security.rs; types.rs and mod.rs referenced where relevant).

**Progress:** Critical and High done (2.1 body conversion, 1.1–1.4, 3.1). Medium and Low done (4.1–4.3, 5.1–5.2). Remaining: optional token/stream parsing (1.3), deeper body handling, types.rs docs.

**Priority key:** Critical = must-have for conversion; High = major quality; Medium = fidelity; Low = polish. Implemented items marked "Done (was X)".

---

## 1. parser.rs

### 1.1 Nested contracts ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~167 | ~~"Nested contracts not yet supported"~~ | **Done:** `Contract` has `nested_contracts: Vec<Contract>`. Parser uses a stack (`nested_stack`, `nested_brace_counts`): when inside a contract we see `contract`/`interface`/`abstract contract`/`library`, push the new contract and track braces; when nested count hits 0, pop and add to parent's `nested_contracts`. Converter flattens via `convert_contract_and_nested`: nested contracts become separate DAL services with names `Parent_Child` (e.g. `Token_ERC20`). |

**Priority:** Done (was High).

---

### 1.2 State variable parsing (simplified) ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~267 | "Parse state variable (simplified - variables without function keyword)" — state vars are detected by: not starting with function/event/modifier/struct/enum/constructor, containing space, not a comment. Then `parse_state_variable(line)` is called. | **Done:** (1) Full type: `last_identifier(before_eq)` finds variable name from the end; type = substring before name with trailing visibility/mutability keywords stripped (public, internal, private, constant, immutable), so types like `mapping(address => uint256)`, `uint256[]` parse correctly. (2) Initializers done. (3) `constant`/`immutable` already set mutability to View and are stripped from type. (4) Name/type extraction is robust for multi-token types. |

**Priority:** Medium — improves fidelity for real contracts.

---

### 1.3 Function parsing: basic implementation ✅ Partial

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~284–286 | "Simplified function parsing. This is a basic implementation - can be enhanced." | **Done (partial):** (1) Multi-line: main loop uses index; when at declaration start (function, constructor, receive, fallback, event, modifier), `get_logical_line_for_declaration` accumulates until balanced `();` or `{...}`; brace counts updated for consumed lines; contract-start line is skipped for parsing. (2) Function body: `parse_function` extracts body between first `{` and matching `}` into `func.body`. (3) Modifiers list, `override`, `virtual` were already parsed. (4) Constructor and receive/fallback already handled. **Remaining:** Token/stream parsing for maximum robustness; full body conversion in generator. |

**Priority:** Partial (was High). Remaining: optional token/stream parsing.

---

### 1.4 Parameter extraction (simplified) ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~326 | "Extract parameters (simplified)" — params string is split by comma; each part split by whitespace; first token = type, second = name. | **Done:** `split_parameters_at_top_level` splits by comma only at depth 0 (not inside `()`, `<>`, `[]`). `parse_parameters` uses last token = name, rest = type, so complex types (`uint256[]`, `mapping(address => uint256)`) parse correctly. Empty param strings (trailing commas) are skipped. |

**Priority:** Medium — required for accurate params when types are complex.

---

## 2. generator.rs

### 2.1 Function body conversion ✅ Done (critical complete)

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~116–122 | ~~"TODO: Convert function body from Solidity"~~ | **Done (critical complete):** (1) require/revert → if/throw. (2) transfer/call→chain::. (3) storage→self.field. (4) emit pass-through. (5) msg.sender→chain::caller(); assignments/return pass-through. (6) **Control-flow pass-through:** `if`/`for`/`while`/`else`/`} else`, `do`/`} while(`, `try`/`catch`, `unchecked`/`assembly` blocks, and lone `{`/`}`. Body has storage→self and msg.sender replaced first. |

**Priority:** Done (was Critical).

---

## 3. analyzer.rs

### 3.1 OpenZeppelin / import check ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~130–131 | ~~"This is a simplified check - in real implementation, would check imports."~~ | **Done:** Parser already has `extract_imports()` → `SolidityAST.imports`. In `analyze()`, check `ast.imports` for "openzeppelin" or "@openzeppelin" and add suggestion: "File imports OpenZeppelin - DAL has built-in reentrancy protection; modifiers can be mapped to @secure". Modifier-based check (`onlyOwner`, `nonReentrant`, `whenNotPaused`) kept for contracts without explicit imports. `AnalysisReport.used_libraries` is populated from `ast.imports` so callers can see which libraries/imports were used. |

**Priority:** Done (was Medium).

---

## 4. converter.rs

### 4.1 Payable mutability ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~136 | ~~"Mutability::Payable => {}, // Handle separately if needed"~~ | **Done:** Generated code includes `// @payable (value handling in DAL may differ)` for payable functions. |

**Priority:** Done (was Low–Medium).

---

### 4.2 Multiple return values ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~154 | ~~"Multiple returns - use tuple or struct (simplified to first return)"~~ | **Done:** `DALFunction` has `comment: Option<String>`. When `func.returns.len() > 1`, set comment to "Multiple returns collapsed to first; original returns (T1, T2, ...)" with converted types. Generator emits comment at start of function body. Return type remains first; full tuple/struct return can be added later. |

**Priority:** Done (was Medium).

---

### 4.3 Reentrancy heuristic ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~182 | ~~"Simple heuristic: if contract has external payable functions, add security"~~ | **Done:** Centralized in security.rs; body-based (.call/.transfer); converter + analyzer use it. |

**Priority:** Done (was Low–Medium).

---

## 5. security.rs

### 5.1 Reentrancy heuristic ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~37 | ~~"Simple heuristic: external payable functions"~~ | **Done:** Aligned with 4.3; security.rs is the single implementation (body-based external-call check). Analyzer uses it for suggestions. |

**Priority:** Low–Medium — same as 4.3.

---

### 5.2 Arithmetic check ✅ Implemented

| Location | Comment / behavior | Real-world direction |
|----------|--------------------|----------------------|
| ~45–46 | ~~"Check if contract uses arithmetic operations. This is a simplified check"~~ | **Done:** `uses_arithmetic` still checks function names (add/sub/mul/div); now also scans function bodies and state-var initializers for `+`, `-`, `*`, `/`, `**` via `body_has_arithmetic` (skips `++`, `--`, `->`). Used for @safe_math in `detect_patterns`. |

**Priority:** Done (was Low).

---

## 6. types.rs (optional)

No explicit "simplified" or "for now" comments; a few behaviors to consider for real-world use:

| Topic | Current behavior | Real-world direction |
|-------|------------------|----------------------|
| Unknown types | Fallback to `string` or pass-through for struct/enum. | Document; optionally map common Solidity types (e.g. `IERC20`) to DAL interface names. |
| Mapping conversion | `convert_mapping` parses `mapping(K => V)`; on parse failure returns `map<string, any>`. | Handle nested mappings and complex key/value types; document fallback. |

**Priority:** Low (optional) — document fallbacks; extend as needed.

---

## Suggested implementation order (by original priority)

| Priority   | Item              | Status | Notes |
|-----------|-------------------|--------|--------|
| **Critical** | 2.1 Body conversion | ✅ Done | require/revert; transfer/call→chain::; storage→self; emit; control-flow; msg.sender; assignments. |
| **High**     | 1.1 Nested contracts | ✅ Done | Parser stack; converter flattens to Parent_Child. |
| **High**     | 1.3 Function parsing | ✅ Partial | Multi-line + body; optional: token/stream. |
| **High**     | 2.1 Storage/emit   | ✅ Done | field_names; word-boundary self; emit pass-through. |
| **Medium**   | 4.2 Multiple returns | ✅ Done | Comment + first return. |
| **Medium**   | 1.2 State variables | ✅ Done | Full type; initializers; constant/immutable. |
| **Medium**   | 1.4 Parameter extraction | ✅ Done | Depth-aware comma; type/name. |
| **Medium**   | 3.1 OpenZeppelin/imports | ✅ Done | used_libraries; suggestions. |
| **Low**      | 4.1 Payable       | ✅ Done | @payable comment in generated code. |
| **Low**      | 4.3, 5.1 Reentrancy | ✅ Done | security.rs; body-based; centralized. |
| **Low**      | 5.2 Arithmetic    | ✅ Done | Body + state-var scan for +,-,*,/,**. |
| **Low**      | 6 types.rs        | Optional | Document fallbacks; extend as needed. |

**Remaining (optional):** Token/stream parsing (1.3); types.rs documentation.

---

## Cross-references

- **Parser plan:** `docs/guides/PARSER_REAL_WORLD_IMPLEMENTATION_PLAN.md` — DAL parser may gain features (e.g. tuples, attributes) that the converter should emit.
- **Runtime plan:** `docs/guides/RUNTIME_REAL_WORLD_IMPLEMENTATION_PLAN.md` — generated DAL must be runnable (function registration, chain:: args, etc.).
- **Migration doc:** `docs/migration/FROM_SOLIDITY.md` — align plan with documented migration path and supported subset.

---

## How to use this plan

- **Priority key:** Critical = must-have; High = major quality; Medium = fidelity; Low = polish. Completed items show "Done (was X)".
- **When implementing:** Find the section above, implement, and remove or update the comment in code.
- **When adding a new placeholder:** Add a comment in code and a new row in this doc with location, comment, and real-world direction.
- **Revisit priority** when expanding supported Solidity subset or improving conversion quality.
