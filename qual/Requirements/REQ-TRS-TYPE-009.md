---
id: REQ-TRS-TYPE-009
type: Requirement
name: Allocation Definition
title: "Tool shall recognise and validate the AllocationDef element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `AllocationDef` definition element. (Its usage `Allocation` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the definition.)

- **SysMLv2 mapping:** `AllocationDef` maps to `allocation def` (§2.1) and classes an allocation relationship. Type-specific fields are defined in §8.13.1: `supertype:`, `ends:` (connection-end schema, §8.4.2), and `allocations:` (nested sub-allocation usages with `allocateFrom:`/`allocateTo:`).
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All §8.13.1 type-specific fields are optional.
- **Recognition behaviour:** a file with `type: AllocationDef` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). An `AllocationDef` with no explicit `supertype:` receives the implicit base-library supertype `Allocations::Allocation` per [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §8.13.1, §11.4

**Acceptance criteria:** A model containing a minimal `AllocationDef` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
