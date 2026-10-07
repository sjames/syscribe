---
type: Requirement
id: REQ-TRS-SYSMLV2-079
name: "assign referent and until loops are native syntax; the step annotation shrinks to what has no syntax"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`assign a.b := v;` shall be ingested as `target: a, referent: b` (a one-segment target has no referent) and `loop { ... } until <c>;` as `loopKind: until, condition: <c>`; `export-sysml` shall write `referent` and `until` as those statements, treating a dotted `target:` without `referent:` as the same value. `@SyscribeStep` is still ingested for backward compatibility and is written only for the fields with no 0.57 syntax: `valueKind` of an assign, and a trigger beside a payload. The repository export ratchet stays 0.
