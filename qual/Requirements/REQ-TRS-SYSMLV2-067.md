---
id: REQ-TRS-SYSMLV2-067
type: Requirement
name: "Equivalent spellings of a transition accept trigger read back as the same value"
status: verified
reqDomain: software
verificationMethod: test
---

A transition `accept:` that is a mapping holding only `payload:` (no `via:`) and the plain string form carry the same meaning, and `syscribe export-sysml` shall treat them as equal when it verifies that an exported transition reads back identically. The canonical stored form (what ingestion produces) is the plain string; the mapping form is `{payload, via}` only when a `via:` is present, and a time trigger (`at`/`when`/`after <expr>`) is written either way. A hand-authored `accept: {payload: X}` shall therefore export as the transition rather than a comment.

**Source:** `REQ-TRS-SYSMLV2-067` (product model), `ADR-SYS-SYSMLV2-002`.
