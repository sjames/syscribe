---
id: REQ-TRS-SYSMLV2-030
type: Requirement
name: Tool shall report, once per SysMLv2 file, an advisory W543 listing per-kind counts of parsed-but-unmapped constructs that ingestion dropped
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall**, for each `.sysml`/`.kerml` file in a `sysmlSubmodel: true` subtree that
contains at least one construct it parses but does not map to a native element (`calc def`,
`constraint`, `use case`, `metadata`, package-level `doc`, and similar), raise exactly one
advisory warning **`W543`** naming the file and the dropped count per construct kind. A file
with no such constructs **shall** raise nothing. The finding is non-blocking: ingestion and the
mapped elements are unchanged.
