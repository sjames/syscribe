---
type: PartDef
name: Downstream
derive:
  c: 'elements["Sys::Xray"].a ?? 7'
---

Depends on a cyclic field but is not part of the cycle: no E504, and c falls back to 7.
