---
type: Requirement
id: REQ-TRS-SYSMLV2-088
name: "export-sysml writes metadata: entries as metadata annotations that read back identically"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`export-sysml` shall write every `metadata:` entry of an exported element inside that element's body as `@T;`, `@T { k = v; }`, `@n : T { ... }` or, for an entry kept with `about:`, `@T about Y;` — integer, real and boolean values bare, strings double-quoted — such that ingesting the exported text reproduces the same `metadata:` entries (with `type:` rewritten to the qualified name of the exported `MetadataDef` when it resolves). The `@Syscribe*` annotations continue to be written from their own native fields, never from `metadata:`.
