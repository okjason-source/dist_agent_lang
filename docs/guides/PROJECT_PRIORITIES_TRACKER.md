# DAL Project Priorities & Progress Tracker

**Created:** 2026-02-05  
**Source:** Strategic planning session  
**Purpose:** Organize and track progress on new plans

---

## Quick Reference

| Priority | Focus | Status |
|----------|-------|--------|
| P0 | Strategic decisions | ✅ Documented |
| **P1** | **CLI expansion** | 🚧 In progress (Phases 0–9) |
| **P2** | Mold marketplace (Plan A) | 🚧 In progress (contract + spec done) |
| P3 | Mold capabilities | 📋 Planned |
| P4 | Fleet composition | 📋 Planned |
| P5 | Supporting work | 📋 Planned |

**Importance order:** CLI expansion first, then Plan A.

**Status legend:** ✅ Done | 🚧 In progress | 📋 Planned | ⏸️ Blocked

---

## P0: Strategic Decisions (Foundation)

*Locked-in decisions that guide all other work.*

| # | Decision | Status | Source |
|---|----------|--------|--------|
| 1 | **Decentralized distribution is Plan A** | ✅ | mold/PLAN_A.md |
| 2 | **Molds** (not "templates") — branded reusable agent configs | ✅ | Same |
| 3 | **Mint fee model** — 30% DAL / 70% creator | ✅ | Same |
| 4 | **Accept crypto** — ETH, USDC; fiat (Stripe) primary | ✅ | Same |
| 5 | **No new token** — DAL will not create a native cryptocurrency | ✅ | Same |
| 6 | **Dynamic NFTs** — molds minted as NFTs that evolve | ✅ | Same |

---

## P1: CLI Expansion

*Highest priority. Practical and optimization commands for all stdlib features.*

| # | Item | Status | Notes |
|---|------|--------|-------|
| 1.1 | Cloud commands (`cloud` not `cloudadmin`) | ✅ | Phase 4 complete |
| 1.2 | Agent commands (`create` not `spawn`) | ✅ | Phase 6 complete |
| 1.3 | Developer tools | ✅ | Phase 0: fmt, lint, check, new, init, repl |
| 1.4 | Optimization commands | ✅ | Phase 1: bench, profile, optimize |
| 1.5 | Domain commands | ✅ | chain, crypto, db, ai, iot (Phase 7), cloud, agent |
| 1.6 | Mold integration | ✅ | `dal agent create --mold <path|ipfs|id>`, mold list/show/create/publish; see [mold/P1_P2_TODO.md](../../../mold/P1_P2_TODO.md) |
| 1.7 | **CLI framework & design** | 🚧 In progress | Phase 11: clap migration (skeleton done); §21 appearance/help next |

**Source:** [09_CLI_EXPANSION_PLAN.md](../development/stdlib_implementation_plans/09_CLI_EXPANSION_PLAN.md)

---

## P2: Mold Marketplace (Plan A Core)

*Second priority. Core product for decentralized distribution.*

| # | Item | Status | Notes |
|---|------|--------|-------|
| 2.1 | Mold marketplace design | ✅ | On-chain; MoldRegistry (mint, useMold, maxUseCount); see mold/contracts |
| 2.2 | Creator minting flow | ✅ | CLI `publish --fee --max-use` (IPFS + mintMold); needs deployed contract |
| 2.3 | Mint fee implementation | ✅ | In contract (30/70); CLI useMold flow done |
| 2.4 | Dynamic mold NFTs | ✅ | mintCount, maxUseCount in contract |
| 2.5 | Metadata API | 📋 | Optional; chain has getMoldInfo |
| 2.6 | CLI: `dal agent create --mold <id> <name>` | ✅ | useMold + load from IPFS; see [mold/P1_P2_TODO.md](../../../mold/P1_P2_TODO.md) |
| 2.7 | **Deploy MoldRegistry** | ⏸️ **Later** | Testnet/mainnet deploy; then set DAL_MOLD_REGISTRY_ADDRESS |

**Source:** [mold/PLAN_A.md](../../../mold/PLAN_A.md)

---

## P3: Mold Capabilities

*What a mold can contain and represent.*

| # | Item | Status | Notes |
|---|------|--------|-------|
| 3.1 | **Molds can have prompts** | 📋 | System prompts, user prompt templates, few-shot examples |
| 3.2 | **Molds can have IoT** | 📋 | Device connectivity, sensor data, MQTT, IoT platform APIs |
| 3.3 | **Fleet as mold** | 📋 | Mold defines fleet composition (multi-agent topology) |
| 3.4 | Mold parameterization | 📋 | Variables (e.g. `{{company_name}}`) filled at mint |
| 3.5 | Mold versioning | 📋 | Creator pushes updates; holders get latest |

**Source:** Conversation 2026-02-05

---

## P4: Fleet Composition (Advanced)

*Adding fleets together; shared state and access control.*

| # | Item | Status | Notes |
|---|------|--------|-------|
| 4.1 | Fleet + fleet composition model | 📋 | Union, chaining, hierarchical |
| 4.2 | **Shared state** across fleets | 📋 | Coordination; consistency; conflict handling |
| 4.3 | **Cross-fleet communication** | 📋 | Addressing, routing, auth |
| 4.4 | **Smart contract access control** | 📋 | Who can join, who can talk to whom, who can read/write state |
| 4.5 | Composition contract | 📋 | Encodes which fleets can be combined |
| 4.6 | State contract | 📋 | Shared state with role/capability-based access |
| 4.7 | Messaging contract | 📋 | Authorizes cross-fleet calls |

**Source:** Conversation 2026-02-05

---

## P5: Supporting Work

*Enables or complements P1–P4.*

| # | Item | Status | Notes |
|---|------|--------|-------|
| 5.1 | Rename "template" → "mold" across docs | 📋 | AGENT_TEMPLATE_MARKETPLACE, CLI plan, etc. |
| 5.2 | Value evolution documentation | ✅ | Single agent → fleet → mold → marketplace |
| 5.3 | Market value context | ✅ | AI agents ~$5–8B; prompt marketplaces ~$1.3B |
| 5.4 | Production roadmap alignment | 📋 | Stdlib editions + Plan A mold marketplace coexist |
| 5.5 | Crypto payment integration | 📋 | ETH, USDC alongside Stripe |

**Sources:** [PRODUCTION_ROADMAP.md](../PRODUCTION_ROADMAP.md)

---

## Related Documents

| Document | Purpose |
|----------|---------|
| [mold/PLAN_A.md](../../../mold/PLAN_A.md) | Plan A strategy; molds; mint fee; dynamic NFTs |
| [AGENT_TEMPLATE_MARKETPLACE.md](./AGENT_TEMPLATE_MARKETPLACE.md) | Marketplace design (rename to molds) |
| [DISTRIBUTION_STRATEGY.md](./DISTRIBUTION_STRATEGY.md) | Binary vs open core; stdlib control |
| [PRODUCTION_ROADMAP.md](../PRODUCTION_ROADMAP.md) | Stdlib editions; v1.1.0 transition |
| [09_CLI_EXPANSION_PLAN.md](../development/stdlib_implementation_plans/09_CLI_EXPANSION_PLAN.md) | CLI commands; cloud, agent, dev tools |

---

## Progress Log

| Date | Change |
|------|--------|
| 2026-02-05 | Created tracker; captured P0–P5 from planning session |
| 2026-02-05 | Reordered: CLI expansion = P1 (highest), Plan A = P2 |
| 2026-02-06 | P1: Phases 7 (iot), 8 (log, config), 9 (bond, pipe, invoke) implemented |
| 2026-02-06 | P1.6 + P2: Mold contract + spec done; [mold/P1_P2_TODO.md](../../../mold/P1_P2_TODO.md) for next steps |
| 2026-02-06 | P1.7: Phase 11 (CLI framework + design) set as next priority |
| 2026-02-06 | P1.7: clap added, skeleton + chain subcommands; all domains wired to handlers |
| 2026-02-06 | P1.6 marked ✅ complete; docs audit (CODEBASE_TODOS, PLANS_AUDIT) |

---

**Next:** P1.7 — Phase 11 CLI framework (clap) + §21 design. Fix `dal --help` panic; custom help (no Phase labels).

**Next review:** Update status as work progresses.
