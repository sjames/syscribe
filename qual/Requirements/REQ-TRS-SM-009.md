---
id: REQ-TRS-SM-009
type: Requirement
name: Tool shall report a state-machine transition that lacks a required source or target (W929)
status: draft
reqDomain: software
verificationMethod: test
---

The transition schema (§8.8.3) makes `target:` **required** on every transition and
`source:` **required** on a **top-level** transition (one listed under the `StateDef`'s /
`State`'s own `transitions:`, where no enclosing substate supplies an implicit source). A
transition missing either endpoint contributes no `(source → target)` edge, so the
completeness checks of §22.1 silently ignore it — a dangling transition could hide a dead or
trap state. The tool **shall** report such a transition:

| Code | Condition |
|---|---|
| `W929` | **Incomplete transition** — a top-level transition has no `source:` (nor its deprecated alias `from:`), or any transition (top-level or nested) has no `target:` (nor `to:`). The finding names the missing endpoint and, when present, the transition's `name:` or its known endpoint. |

- `W929` belongs to the state-machine completeness family (`W070`–`W079`): it is a warning,
  **draft-suppressed** (not emitted for `status: draft`), and **gateable** with
  `--deny W929`. It fires whether or not the machine declares `subStates:`.
- A **nested** transition (under a `subStates:` entry's own `transitions:`) with no
  `source:` is **not** reported — the enclosing substate is its implicit source.

**Source:** GH #136 (v0.40.1 documentation/spec consistency review); spec §8.8.3, §22.1.

**Acceptance criteria:**

- A top-level transition with a `target:` but no `source:` raises `W929` naming the missing
  `source`.
- A nested transition with no `target:` raises `W929` naming the missing `target`.
- A nested transition with no `source:` (implicit source) raises no `W929`.
- `W929` is suppressed for `status: draft` and gates non-zero under `--deny W929`.
- The shipped models are `W929`-clean.
