---
id: REQ-TRS-DERIVE-004
type: Requirement
name: "Derive engine shall detect and report cyclic dependencies as E504"
status: draft
reqDomain: software
verificationMethod: test
---

Before evaluating any derived field, the tool **shall** build a dependency graph across all `derive:` blocks in the model and check it for cycles.

A cycle occurs when field A on element X depends (directly or transitively) on field A (or another derived field on X) through an element chain that leads back.

When a cycle is detected, the tool **shall** emit error **`E504`** naming the cycle, and skip evaluation of all fields participating in it.

`E504` was previously reserved as `E500`, which is the Allocation `allocatedFrom:` resolution error (REQ-TRS-VAL-009); the derive
codes were moved to `E504`–`E506` so each code carries exactly one meaning (GH #127). Cycle detection itself is not yet
implemented — `E504` stays reserved for it.

**Acceptance criteria:**

- A self-referential formula (`fieldA: self.fieldA + 1`) triggers E504.
- Two elements mutually depending on each other's derived fields triggers E504.
- A valid chain (A depends on B, B depends on C) evaluates correctly without E504.
