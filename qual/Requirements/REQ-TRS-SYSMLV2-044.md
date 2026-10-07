---
id: REQ-TRS-SYSMLV2-044
type: Requirement
name: Tool shall ingest a SysMLv2 library package or namespace as a Package, at the root and nested
status: verified
reqDomain: software
verificationMethod: test
---

`library package` and `namespace` declarations shall be merged and converted exactly like a
`package`: a named one becomes a `Package` element with its members nested under it (same-named
declarations across files merge), at the root of a file and nested inside another package. The
`standard` marker of a `library package` is not preserved. Neither kind counts toward `W543` any
more.

**Source:** `REQ-TRS-SYSMLV2-044` (product model), `ADR-SYS-SYSMLV2-001`.
