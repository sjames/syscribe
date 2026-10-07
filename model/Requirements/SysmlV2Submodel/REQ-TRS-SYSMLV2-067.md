---
type: Requirement
id: REQ-TRS-SYSMLV2-067
name: "Equivalent spellings of a transition accept trigger read back as the same value"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

A transition `accept:` that is a mapping holding only `payload:` (no `via:`) and the plain string form carry the same meaning, and `syscribe export-sysml` shall treat them as equal when it verifies that an exported transition reads back identically. The canonical stored form (what ingestion produces) is the plain string; the mapping form is `{payload, via}` only when a `via:` is present, and a time trigger (`at`/`when`/`after <expr>`) is written either way. A hand-authored `accept: {payload: X}` shall therefore export as the transition rather than a comment.
