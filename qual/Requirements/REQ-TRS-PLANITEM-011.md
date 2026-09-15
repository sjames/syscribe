---
id: REQ-TRS-PLANITEM-011
type: Requirement
name: Tool shall provide advisory claim/release markers on PlanningItem
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support two optional fields on `PlanningItem`, `claimedBy:` (an opaque
claimant id) and `claimedAt:` (an opaque, conventionally ISO-8601 timestamp — never itself
format-validated), and two commands to manage them:

- `syscribe claim <PI-id> --by <agent-id>` **shall** set both fields. It **shall** refuse (no
  file written) when the target is already `status: done` (nothing to claim), or when
  `claimedBy:` is already set to a value other than `<agent-id>`. Re-claiming with the same
  `<agent-id>` **shall** be allowed and refresh `claimedAt:`.
- `syscribe release <PI-id>` **shall** clear both fields, unconditionally, regardless of the
  item's current `status:`. It **shall** be a no-op (nothing written) when the item was not
  claimed.

Both commands **shall** resolve their target by qualified name or stable id, refuse when the
target is not a `PlanningItem`, and support `--dry-run` (preview the unified diff without
writing). `claimedBy` **shall** be visible in `show <PI-id>` and in
`list PlanningItem --json`.

**Motivation:** running several LLM agents concurrently against one model, each independently
implementing a different `PlanningItem`, the only thing preventing two agents from editing the
same files at once was a rule held in an orchestrator's own memory ("never start a new agent
whose file scope overlaps a currently-running one"). This gives the tool visibility into "is
anything already being worked" instead of requiring an orchestrator to reconstruct it from
`git status`, running-process lists, or its own memory of what it dispatched.

**Source:** GitHub issue #115, `ADR-SYS-PLANITEM-001` (addendum).

**Acceptance criteria:**
- `claim` sets `claimedBy`/`claimedAt`; refuses clearly (no write) if claimed by someone else.
- `claim` on an already-`done` item is rejected (nothing to claim).
- `release` clears both fields regardless of current `status`; a no-op when unclaimed.
- `claimedBy` is visible in `list PlanningItem --json` and in `show <id>`.
- `--dry-run` previews without writing, for both commands.
