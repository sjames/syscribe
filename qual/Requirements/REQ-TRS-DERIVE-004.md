---
id: REQ-TRS-DERIVE-004
type: Requirement
name: "Derive engine shall detect and report cyclic dependencies as E500"
status: draft
reqDomain: software
verificationMethod: test
---

Before evaluating any derived field, the tool **shall** build a dependency graph across all `derive:` blocks in the model and check it for cycles.

A cycle occurs when field A on element X depends (directly or transitively) on field A (or another derived field on X) through an element chain that leads back.

When a cycle is detected, the tool **shall** emit error **`E500`** naming the cycle, and skip evaluation of all fields participating in it.

**Acceptance criteria:**

- A self-referential formula (`fieldA: self.fieldA + 1`) triggers E500.
- Two elements mutually depending on each other's derived fields triggers E500.
- A valid chain (A depends on B, B depends on C) evaluates correctly without E500.
