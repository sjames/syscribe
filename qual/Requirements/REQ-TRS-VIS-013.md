---
id: REQ-TRS-VIS-013
type: Requirement
name: The legacy renderer, manifest helpers and CLI diagram toolkit are removed outright
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall not** offer the former CLI `diagram` toolkit (`list`, `render`, `measure`,
`compose`, `layout`, `seq`, `req`): each of those **shall** be rejected as an unrecognized
subcommand of `diagram`, and the `diagram` man page **shall** describe only `diagram export`
(`REQ-TRS-VIS-009`, the one `diagram` subcommand that exists after `ADR-SYS-VIS-001`). The
`syscribe-model` crate **shall** no longer export a `renderer` or `diagram` module; every diagram
consumer reads the Diagram IR (`vis`). Hand-authored `diagramKind: Mermaid` and inline
`diagramKind: PlantUML` diagrams, the `render <path>` command and `syscribe plantuml` **shall**
be unaffected.

**Source:** `REQ-TRS-VIS-013` (product model); user decision of 2026-10-07 that no backwards
compatibility is owed to any rendering path. Between Phase 0 and Phase 3 no `diagram` command
existed at all (`help diagram` exited non-zero); Phase 3 re-introduced `diagram export` only.

**Acceptance criteria:** `syscribe -m <root> diagram list` (and `measure`, `compose`, `layout`,
`seq`, `req`) exits non-zero with an "unrecognized subcommand" message and nothing on stdout;
`syscribe help diagram` prints a `SYNOPSIS` that offers `diagram export` and none of the retired
subcommands; `syscribe help render` and `syscribe help plantuml` still print a `SYNOPSIS`; a
`Mermaid`-kind diagram in a model still validates with no error.
