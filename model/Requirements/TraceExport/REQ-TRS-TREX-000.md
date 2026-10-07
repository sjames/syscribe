---
type: Requirement
id: REQ-TRS-TREX-000
name: "A single JSON export reports every requirement's full traceability — derivation, satisfaction and verification — for coverage checks"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - traceability
  - export
---

Syscribe shall produce, in one command, a single JSON document that lists every requirement in
the model with its complete traceability: the requirements it derives from and those derived
from it, the architecture elements that satisfy it and the test cases that verify it, every
element named by its full qualified name, so that requirement coverage can be checked by a
script or an external tool without reconstructing links from the whole-model dump. The report
shall optionally be restricted to the elements active in a product-line configuration, and shall
accept a sort order: the model's directory order, ascending or descending by qualified name.

## Rationale

Coverage is the question every safety and systems review starts with, and today it takes a
per-requirement command or a hand-written join over `export`. One document with the same
numbers `validate` enforces makes coverage checkable in CI and consumable by other tools.

## Scope

- In scope: the document, its configuration projection, its sort orders, CLI and MCP surfaces.
- Out of scope: editing links, rendering (the Requirement diagram covers that), and HTML/CSV
  renderings of the same data.
