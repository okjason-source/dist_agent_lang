# Workflows in dist_agent_lang

How to run multi-step agent workflows and how this relates to the `ai::` namespace.

---

## Production path: use `workflows.dal` and `wf::run(...)`

For **multi-step agent workflows** (research → summarize, code → review, etc.), use the **DAL workflow layer**:

1. **Import the workflow module** (e.g. from your agent or app):
   ```dal
   import "./workflows.dal" as wf;
   ```

2. **Run a built-in or custom workflow by name**:
   ```dal
   let result = wf::run("research_and_summarize", "latest trends in AI agents");
   let result = wf::run("code_and_review", "implement a retry helper");
   ```

3. **Built-in workflows** (see `workflows.dal` in your project or `agent_assistant/workflows.dal`):
   - `research_and_summarize` — researcher gathers info, reviewer summarizes
   - `code_and_review` — coder writes code, reviewer checks quality
   - `deep_research` — researcher → coder (commands) → reviewer (report)

4. **Custom workflows** can be defined via the API (e.g. `POST /api/workflow/define` with a steps array) and are then runnable by name via `wf::run(name, input)`.

This layer uses `agents::run_with_agent(role, prompt)` under the hood and returns structured results (`ok`, `workflow`, `steps`, `final_result`). This is the **supported, production-ready** way to run workflows in DAL.

---

## `ai::` workflow APIs: coordinator / workflow / execute (wired in runtime)

The **standard library** in `ai.rs` defines a lower-level workflow model:

- **Types:** `AgentCoordinator`, `Workflow`, `WorkflowStep`, `WorkflowStatus`, `StepStatus`
- **Functions:** `ai::create_coordinator`, `ai::add_agent_to_coordinator`, `ai::create_workflow`, `ai::execute_workflow`, `ai::get_coordinator_metrics`, etc.

The **DAL runtime** wires these to the real implementation:

1. **ai::spawn_agent(config)** — Creates an `Agent` and stores it for use with coordinators. Pass a map/struct with `name`, `role`, optional `capabilities`.
2. **ai::create_coordinator(coordinator_id)** — Creates a coordinator and stores it; returns the id.
3. **ai::add_agent_to_coordinator(coordinator_id, agent_id)** — Adds an agent (from spawn_agent) to the coordinator.
4. **ai::create_workflow(coordinator_id, name, steps)** — Adds a workflow. `steps` is a list of maps with `step_id`, `agent_id`, `task_type`, optional `dependencies`.
5. **ai::execute_workflow(coordinator_id, workflow_id)** — Runs the workflow (steps in dependency order, tasks on agents); returns a boolean.
6. **ai::get_coordinator_metrics(coordinator_id)** — Returns a map with `agents_count`, `workflows_count`, etc.

Use **workflows.dal** and **wf::run(...)** for the higher-level, role-based chains (research_and_summarize, code_and_review, etc.). Use **ai::** when you need explicit coordinator/workflow/step control and dependency ordering.

---

## Summary

| Need | Use |
|------|-----|
| Multi-step agent workflows (research, code+review, custom chains) | **workflows.dal** + **wf::run(workflow_name, input)** |
| Low-level coordinator/workflow/step control (steps, dependencies, explicit agents) | **ai::** — create_coordinator, spawn_agent, add_agent_to_coordinator, create_workflow, execute_workflow |

See also: [Production grade checklist](PRODUCTION_GRADE_CHECKLIST.md), [Agent Setup and Usage](guides/AGENT_SETUP_AND_USAGE.md).
