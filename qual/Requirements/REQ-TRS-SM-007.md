---
id: REQ-TRS-SM-007
type: Requirement
name: Tool shall flag transitions whose endpoints do not resolve to a state in scope (W076)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit **`W076`** for any state-machine transition whose `source` or
`target` endpoint resolves to **no state** — i.e. the endpoint name is **not** the name of a
state anywhere in the same machine's hierarchy **and** does not resolve to a model element by
qualified name (via the standard resolver). This catches typos and references to removed
states.

- The state-name "universe" for a machine **shall** be the set of all substate names at
  every nesting level (top, parallel regions, and composite interiors), so a legitimate
  cross-level transition that names a state elsewhere in the same machine does **not** raise
  `W076`.
- An endpoint that resolves to a known model element by qualified name (e.g. a fully
  qualified state reference) **shall not** raise `W076`.
- `W076` **shall** be **draft-suppressed** and **gateable** with `--deny W076`, consistent
  with the other state-machine warnings. Each unresolved endpoint name is reported once per
  element.

**Source:** SysMLv2 §7.18 (transition `source`/`target` must denote states); endpoint
integrity for the hierarchy-aware checks of [[REQ-TRS-SM-006]].

**Acceptance criteria:**

- A transition whose `target` names a non-existent state raises `W076`.
- A transition whose endpoints are all states in the machine (at any level) raises no
  `W076`.
- `W076` is suppressed for `status: draft` and gates non-zero under `--deny W076`.
- The shipped models (`model`, `model_mg`, `model_sil`, `model_auto`) are `W076`-clean.
