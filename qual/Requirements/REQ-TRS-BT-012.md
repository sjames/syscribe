---
id: REQ-TRS-BT-012
type: Requirement
name: "A staged Configuration binds only variability whose window contains its stage"
status: draft
reqDomain: software
verificationMethod: test
---

A staged Configuration shall select a feature or bind a parameter only if the item's window contains the Configuration's stage; otherwise it shall be an error (`E752`): for a feature, each entry of its own `features:` map is checked against the feature's window; for a parameter, each `parameterBindings:` entry — including one that reaches a `subConfigurations:` tier — against the parameter's window. A runtime parameter bound at stage `runtime` is within its window. For an unstaged Configuration, `W027` (binding a `runtime` parameter) shall be retained as in [[REQ-TRS-PARAM-004]].

**Acceptance criteria:**
- A `compile`-stage Configuration binding a parameter whose window starts at `load` is an error.
- An unstaged Configuration binding a `runtime` parameter still produces `W027`.
