---
id: REQ-TRS-TYPE-002
type: Requirement
name: "Tool shall recognise and validate the CalculationDef and Calculation elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `CalculationDef` definition element and its `Calculation` usage.

- **SysMLv2 mapping:** `CalculationDef` maps to `calc def` and `Calculation` to `calc` (§2.1, §2.2). A `CalculationDef` is a parameterized expression that returns a value; a `Calculation` is a usage that invokes one. Type-specific fields are defined in §8.9: on `CalculationDef` — `parameters:` (action-parameter schema, §8.7.2), `returnType:`, `body:`, `bodyLanguage:`; on `Calculation` — `typedBy:` (the CalculationDef), `multiplicity:`, `bindingConnections:`.
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization (CalculationDef) and `typedBy:` for typing (Calculation). They carry **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. All §8.9 type-specific fields are optional.
- **Recognition behaviour:** a file with `type: CalculationDef` or `type: Calculation` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). A `CalculationDef` with no explicit `supertype:` receives the implicit base-library supertype `Calculations::Calculation` per [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.1, §2.2, §8.9, §11.4

**Acceptance criteria:** A model containing a minimal `CalculationDef` and a `Calculation` typed by it parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
