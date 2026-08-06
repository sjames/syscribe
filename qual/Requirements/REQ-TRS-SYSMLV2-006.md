---
id: REQ-TRS-SYSMLV2-006
type: Requirement
name: A SysMLv2 ingestion failure shall degrade gracefully and never abort validation
status: draft
reqDomain: software
verificationMethod: test
---

A malformed `sysmlSubmodel:` declaration, a `.sysml`/`.kerml` parse failure, or an unmapped
construct **shall** never abort the rest of `validate`. Each failure **shall** downgrade only the
affected file's (or subtree's) contribution — fewer or no elements from that file, plus a
`Finding` in the normal validate report — while every other native and SysMLv2-originated element
in the model validates normally. This subsystem **shall** use its own dedicated error/warning
code range, distinct from any sandboxed foreign-format-plugin code range.

**Source:** `REQ-TRS-SYSMLV2-006` (product model).

**Acceptance criteria:** `sysmlSubmodel:` set to a non-boolean YAML value degrades via the tool's
normal malformed-frontmatter handling (a `Finding` naming that file), with the rest of the model —
including sibling packages — validating normally and the process never crashing; a `.sysml` file
with a syntax error produces a `Finding` naming that file and contributes zero elements, without
aborting validation of the rest of the model; a construct outside the mapped element set (a
`state`/`action` body) parses without error and contributes zero elements, without any `Finding`
at all.
