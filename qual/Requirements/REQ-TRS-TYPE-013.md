---
id: REQ-TRS-TYPE-013
type: Requirement
title: "Tool shall recognise and validate the Metadata element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `Metadata` usage element. (Its definition `MetadataDef` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the usage.)

- **SysMLv2 mapping:** `Metadata` maps to `metadata` (§2.2) and is the application of a `MetadataDef` (`metadata def`, §8.15). The application form is most often the inline `metadata:` field on a host element (§8.15.2 / §3.8), where each entry is a map with a required `type:` (the MetadataDef qualified name) plus attribute values; the standalone file form carries `type: Metadata` with `typedBy:` referencing the MetadataDef.
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `typedBy:` for typing. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem.
- **Recognition behaviour:** a file with `type: Metadata` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application for `MetadataDef` (`Metadata::SemanticMetadata`) follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.2, §3.8, §8.15.2

**Acceptance criteria:** A model containing a minimal `Metadata` usage (with its `MetadataDef`) parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
