---
id: REQ-TRS-DERIVE-004
type: Requirement
name: "Derive engine shall detect and report cyclic dependencies as E504"
status: draft
reqDomain: software
verificationMethod: test
---

Before evaluating any derived field, the tool **shall** build a dependency graph across all `derive:` blocks in the model and check it for cycles.

A node of the graph is one derived field (element, field name). A field depends on another derived field when its formula reads it via `self.<field>`, `elements["QName"].<field>`, or a `children`/`parent` aggregate over `<field>` whose member declares `<field>` in its own `derive:` block. A cycle occurs when field A on element X depends (directly or transitively) on itself — a self-reference, a loop within X's block, or a chain through other elements that leads back.

When a cycle is detected, the tool **shall** emit error **`E504`** naming the cycle (each participating `element.field` in dependency order), once on every element file that takes part in it, and skip evaluation of all fields participating in it (they are absent from the element's derived fields). Fields outside the cycle — including ones that depend on a cyclic field — are still evaluated; a reference to a skipped field reads as absent.

`E504` was previously reserved as `E500`, which is the Allocation `allocatedFrom:` resolution error (REQ-TRS-VAL-009); the derive
codes were moved to `E504`–`E506` so each code carries exactly one meaning (GH #127). Cycle detection is implemented by
GH #141 (previously `E504` was reserved and a cyclic `derive:` graph was not reported).

**Acceptance criteria:**

- A self-referential formula (`fieldA: self.fieldA + 1`) triggers E504.
- Two elements mutually depending on each other's derived fields triggers E504.
- A valid chain (A depends on B, B depends on C) evaluates correctly without E504.
