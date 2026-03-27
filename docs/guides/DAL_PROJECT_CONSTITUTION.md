# DAL Project Constitution 

**Lead by OKJason aka Jason Dinh - The Chief Creator, Principle and King of the DAL Technosphere**

**Status:** Active  
**Scope:** Entire `dist_agent_lang` ecosystem (language, libraries, runtime, IDE, agents, reusable intelligence, distribution)

---

## Why This Exists

DAL is not only a programming language. It is a system for getting real work done through agentic orchestration while preserving developer-grade control.

This constitution defines the project shape so product decisions, implementation choices, and roadmap priorities stay coherent as the platform expands.

---

## Core Thesis

DAL is a **work language**:

- A language where users can start simple and progressively unlock deeper capability.
- A runtime where agents can reason, act, and complete tasks through host-executed tools.
- A platform where reusable intelligence (skills, molds, oracles) can be created, shared, governed, and even monetized.

---

## User Continuum (No Rigid Personas)

DAL does **not** assume fixed categories like "builder", "operator", or "end user".

Any person can move along a continuum by context and skill:

- Ask and instruct and use
- Configure and compose
- Build and publish
- Operate and govern

The same interface and model should support this progression without forcing a product switch.

---

## Constitutional Principles

1. **One Surface, Progressive Depth**  
   Defaults must be safe and intuitive; advanced controls must be discoverable without requiring a separate product.

2. **Host-Executed Truth**  
   Critical actions are executed by typed host tools and policy gates, not inferred from model prose.

3. **DAL-First Orchestration**  
   Application orchestration belongs in DAL (`*.dal`), while provider protocol details and enforcement stay in runtime/stdlib.

4. **Real Work Completion Over Demo Behavior**  
   Agent quality is measured by completed outcomes with evidence, retries, and clear terminal states.

5. **Operational Safety by Default**  
   Bounded loops, guardrails, diagnostics, and rollback controls are mandatory for production surfaces.

6. **Open Composition Economy**  
   Skills, molds, and oracles are first-class reusable units with transparent packaging, policy, versioning, and licensing.

7. **Same Capability Across Surfaces**  
   CLI, server, and IDE should converge on behavior and diagnostics; differences should be explicit and intentional.

8. **Evidence Before Claims**  
   Tool results and observable outputs are the basis for system claims, especially in validation and incident triage.

9. **Secure by Construction**  
   Capability boundaries, secrets handling, and side-effect controls are baked into architecture, not bolted on later.

10. **Human Agency Preserved**  
    Automation should increase user leverage while keeping clear override and approval pathways.

---

## Product Shape

DAL evolves as a layered system:

- **Language layer:** syntax, types, module system, compile targets.
- **Runtime layer:** deterministic execution, stdlib contracts, safety controls.
- **Agent layer:** planning, tool use, memory/context, orchestration loops.
- **Distribution layer:** skills/molds/oracles packaging, sharing, licensing, governance.
- **Experience layer:** CLI, IDE, server APIs, and guided onboarding.

Each layer should strengthen the others; no layer should require users to abandon prior workflows.

---

## Production Readiness Definition (DAL)

A DAL surface is production-ready when it satisfies all:

- **Usability:** sane defaults with a clear first-run path.
- **Reliability:** bounded execution with predictable terminal states.
- **Safety:** explicit policy gates for side effects.
- **Observability:** actionable diagnostics and metrics for operators.
- **Portability:** local and hosted deployments behave consistently.
- **Extensibility:** users can compose and publish capabilities without breaking core safety.

---

## Strategic Outcomes This Constitution Supports

1. A user can start by asking for help and quickly graduate to composing workflows.
2. A developer can package a skill/mold/oracle and distribute it with clear rights and constraints.
3. A team can run DAL locally or hosted (for example, AWS IDE backend) with the same safety and telemetry model.
4. Agent behavior can be validated end-to-end with evidence, not assumptions.
5. The platform can scale economically without fragmenting user experience.

---

## Guardrails for Future Decisions

Before accepting major feature work, answer:

1. Does this improve real task completion, or only add novelty?
2. Does it preserve progressive depth in the same surface?
3. Does it strengthen host-executed safety and observability?
4. Does it keep DAL-first orchestration intact?
5. Does it help a broader range of users move forward, regardless of starting skill?

If most answers are "no", the work is likely out of constitutional alignment.

---

## Execution Commitments

To keep this constitution actionable, DAL will maintain:

- A living roadmap tied to these principles.
- A priorities tracker that maps every major initiative to constitutional outcomes.
- Validation runbooks proving agent task completion in realistic workflows.

Related planning anchors:

- `docs/PRODUCTION_ROADMAP.md`
- `docs/guides/PROJECT_PRIORITIES_TRACKER.md`
- `docs/development/AGENT_HOST_PROTOCOL_PLAN.md`
