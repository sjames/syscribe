---
type: Requirement
id: REQ-TRS-AGENTPOL-004
name: "Questions reserved to a person are not answered by agents"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - agents
  - planning
---

A PlanningItem question whose decision belongs to a person shall not be answered by an agent, and an answer an agent wrote to such a question shall be reported (GH #457).

## Behavior

- A question may declare `answerBy: human | any`. `[agents] human_only_answers = true` makes `human` the default for every question; a question's own `answerBy` always wins over the default, so `answerBy: any` opens one question to agents under the policy and `answerBy: human` reserves one without any `[agents]` table. An unrecognised value is `E724` and counts as `human`. A malformed `human_only_answers` is `W762`.
- An agent's change (the same rule as the rest of the policy: `--by` without `--as-human`, or any MCP write) that sets a reserved question's `answer`, `answeredBy` or `answeredAt`, moves it to `answered`, creates it already answered, or lifts its reservation is refused with the rule `human_only_answers` named — exit 1 on the CLI, also under `--dry-run`, nothing written. The check runs in the CLI write layer and in the MCP guarded write, so `set … question.answer`, the MCP `question`, `update_element` and `apply_changes` tools, and any other writer, are all held to it. Recording a question, editing its text, reopening or deferring it stay allowed.
- `validate` raises `W323` for a reserved, answered question when its `answeredBy` matches `[agents] actors`, or when the git history (the change log, GH #443) shows the commit that last changed the answer was an agent's (a `Syscribe-Actor` trailer without `Syscribe-As: human`, or an author matching `actors`). An answer whose writer cannot be told — not in git, not committed yet — raises nothing.
- `questions --json` (and the MCP `questions` tool) gives each question `answerBy` (`human` | `any`, the default applied) and, for an answered question whose writer git records, `answerProvenance: {by, agent, at, commit}`. The text table marks reserved questions.
