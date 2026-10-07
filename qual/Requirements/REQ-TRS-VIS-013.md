---
id: REQ-TRS-VIS-013
type: Requirement
name: The legacy renderer, manifest helpers and CLI diagram toolkit are removed outright
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall not** offer the former CLI `diagram` toolkit (`list`, `render`, `measure`,
`compose`, `layout`, `seq`, `req`): `syscribe diagram ...` **shall** be rejected as an unknown
subcommand and `syscribe help diagram` **shall** exit non-zero with no man page. The
`syscribe-model` crate **shall** no longer export a `renderer` or `diagram` module; every diagram
consumer reads the Diagram IR (`vis`). Hand-authored `diagramKind: Mermaid` and inline
`diagramKind: PlantUML` diagrams, the `render <path>` command and `syscribe plantuml` **shall**
be unaffected.

**Source:** `REQ-TRS-VIS-013` (product model); user decision of 2026-10-07 that no backwards
compatibility is owed to any rendering path.

**Acceptance criteria:** `syscribe -m <root> diagram list` exits non-zero with an
"unrecognized subcommand" message and nothing on stdout; `syscribe help diagram` exits
non-zero and prints no `SYNOPSIS`; `syscribe help render` and `syscribe help plantuml` still
print a `SYNOPSIS`; a `Mermaid`-kind diagram in a model still validates with no error.
