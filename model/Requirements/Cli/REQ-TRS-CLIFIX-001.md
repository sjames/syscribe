---
type: Requirement
id: REQ-TRS-CLIFIX-001
name: "CLI output defects: behavioral-coverage header, external implementedBy targets, path-form scopes"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
---

Three CLI output defects (GH #231) shall be corrected.

## Behavior

- `behavioral-coverage` without a scope shall print a real label (`whole model`), never the literal placeholder `<model>`.
- `links <element>` shall show target type `external` for an `implementedBy` value that is a remote URI or package-registry reference, matching what `validate` accepts, instead of `(unresolved)`.
- `ls <path>` given a `/`-separated scope that is not a qualified name shall accept it as the `::` form when that resolves to a package, or otherwise print `did you mean <A::B>?`.
