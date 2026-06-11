---
id: REQ-TRS-TYPE-015
type: Requirement
name: Library Package and Namespace
title: "Tool shall recognise and validate the LibraryPackage and Namespace elements"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `LibraryPackage` and `Namespace` namespace elements. (The ordinary `Package` is covered by [[REQ-TRS-ELEM-001]]; this requirement specifies the two remaining namespace types.)

- **SysMLv2 mapping:** `LibraryPackage` maps to `library package` — a package marked as a model library; `Namespace` maps to the implicit root namespace, carried by the `_index.md` at the model root with no explicit parent (§2.3, §8.1). Both are named containers for model elements and live as directory `_index.md` files (or as stand-alone package files).
- **Identity style:** both are **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar). The root `Namespace` has the empty qualified name. They carry **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the directory/filename stem.
- **Recognition behaviour:** a file with `type: LibraryPackage` or `type: Namespace` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-PARSE-001]].

**Source:** §2.3, §8.1

**Acceptance criteria:** A model whose root `_index.md` is `type: Namespace` and which contains a `type: LibraryPackage` subpackage parses both at their declared types (visible in `export`) and raises no `E005` for either; a sibling file with an unrecognised `type:` value raises `E005`.
