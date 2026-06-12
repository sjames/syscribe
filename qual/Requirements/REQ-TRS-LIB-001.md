---
id: REQ-TRS-LIB-001
type: Requirement
title: Tool shall recognise the auto-imported standard-library types and flag unknown members of them
status: draft
reqDomain: software
verificationMethod: test
---

The two **auto-imported** SysMLv2 standard-library packages have a **fully known**
membership (spec §4.7) and **shall** be recognised by the tool as resolvable type
references, and — because their membership is known — a reference to a **non-existent
member** of one of them **shall** be flagged as a likely typo.

### Recognised built-in types

- **`ScalarValues`** — `Integer`, `Real`, `Natural`, `Boolean`, `String`.
- **`Base`** — `Anything`, `DataValue`.

### Validation

- A type reference (`supertype:`, `typedBy:` — including nested in `features`, `ports`,
  `operations`, and `parameters` — `returnType:`, and a `parameters[].type`) to a
  **known** built-in member (e.g. `ScalarValues::Real`, `Base::DataValue`) **shall**
  resolve cleanly: it **shall not** raise `W404` or any unresolved-reference finding.
- A reference of the form `<Pkg>::<member>` where `<Pkg>` is one of the known
  auto-imported packages (`ScalarValues`, `Base`) but `<member>` is **not** one of that
  package's known members (e.g. `ScalarValues::Flota`, `Base::Nope`) **shall** raise
  warning **`W043`**, naming the offending member and listing the package's known
  members. This applies in every type-reference context above (it catches typos that are
  otherwise silently accepted).
- **Import-only** standard-library packages whose membership is **not** enumerated
  (`SI`, `ISQ`, `Parts`, …) remain **lenient**: a reference into them is **not** subject
  to `W043` (an unresolved `typedBy:`/`returnType:` is at most the existing `W404`). The
  member-typo check applies only to the fully-known auto-imported packages.
- `W043` is a **warning** (advisory, gateable with `--deny W043`), consistent with the
  lenient treatment of standard-library type references.

**Source:** user request — make the built-in `ScalarValues`/`Base` types *recognised*
(resolve with no `W404`) and a typo against them *flagged*. Refines the standard-library
inventory (spec §4.7) and the type-reference resolution behind `W404`.

**Acceptance criteria:**

- `typedBy: ScalarValues::Real` (and `Integer`/`Natural`/`Boolean`/`String`) and
  `Base::DataValue` validate with **no** `W404` and **no** `W043`, including as an
  operation `returnType`/parameter `typedBy` (where a valid member previously raised
  `W404`).
- `typedBy: ScalarValues::Flota` raises `W043` naming `Flota` and the known
  `ScalarValues` members; the same applies in `supertype:`, `returnType:`, and a
  `parameters[].type`.
- `Base::Nope` raises `W043`; a valid in-model or recognised reference does not.
- `typedBy: SI::kg` raises **no** `W043` (import-only package, membership not enumerated).
