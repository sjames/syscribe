---
type: Argument
id: ARG-BAD-001
name: "Argument with an unresolved supports ref"
status: approved
argumentType: strategy
supports: SG-DOES-NOT-EXIST-999
evidence:
  - REQ-MISSING-001
---

`supports` and `evidence` both name elements that do not exist, so E855 fires.
