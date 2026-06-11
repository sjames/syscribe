---
id: REQ-TRS-TYPE-016
type: Requirement
name: Dependency Relationship
title: "Tool shall recognise and validate the Dependency element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `Dependency` relationship element.

- **SysMLv2 mapping:** `Dependency` maps to `dependency` (§2.4) — a directed relationship from one or more client elements to one or more supplier elements. Unlike definitions and usages it is not a classifier or feature: it lives at package level, has no `typedBy:`, and cannot be owned as a sub-feature. Type-specific fields (§2.4, §3.9): `clients:` (required) and `suppliers:` (required) lists of qualified names; specializations such as `Realization`/`Derivation` are expressed via `supertype:` referencing library types. The `dependsOn:` field on any element is the inline client-side equivalent (§3.9).
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar), with `supertype:` for specialization. It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `clients:` and `suppliers:` are required; `name:` is optional and defaults to the filename stem.
- **Recognition behaviour:** a file with `type: Dependency` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-XREF-001]].

**Source:** §2.4, §3.9

**Acceptance criteria:** A model containing a minimal `Dependency` with valid `clients:`/`suppliers:` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
