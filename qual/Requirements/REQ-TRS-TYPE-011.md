---
id: REQ-TRS-TYPE-011
type: Requirement
name: Rendering Definition
title: "Tool shall recognise and validate the RenderingDef element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `RenderingDef` definition element.

- **SysMLv2 mapping:** `RenderingDef` maps to `rendering def` (§2.1) and classifies a rendering method for views. Its schema (§8.14.4) is the common definition schema: `supertype:` and owned `features:`. The corresponding usage is `Rendering` (`rendering`, §2.2).
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. `features:` and `supertype:` are optional.
- **Recognition behaviour:** a file with `type: RenderingDef` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). A `RenderingDef` with no explicit `supertype:` receives the implicit base-library supertype `Views::Rendering` per [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §8.14.4, §11.4

**Acceptance criteria:** A model containing a minimal `RenderingDef` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
