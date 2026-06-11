---
id: REQ-TRS-TYPE-010
type: Requirement
title: "Tool shall recognise and validate the SuccessionDef element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `SuccessionDef` definition element.

- **SysMLv2 mapping:** `SuccessionDef` maps to `succession def` (§2.1) and classifies temporal ordering between occurrences. The corresponding usage `Succession` (`succession`, §2.2) expresses ordering between actions/occurrences; succession ordering inside a behavioural element is also expressed inline via the `successionConnections:` field (`after:`/`before:`/`guard:`/`effect:`, §8.4.4).
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem.
- **Recognition behaviour:** a file with `type: SuccessionDef` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §2.2, §8.4.4

**Acceptance criteria:** A model containing a minimal `SuccessionDef` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
