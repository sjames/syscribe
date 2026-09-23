---
id: REQ-TRS-LINKTYPE-001
type: Requirement
name: "A project shall be able to declare user-defined link types in a [linkTypes] table of .syscribe.toml, with malformed declarations reported and ignored"
status: draft
reqDomain: software
verificationMethod: test
---

A project **shall** be able to declare link types in `<model_root>/.syscribe.toml` as
`[linkTypes.<name>]` tables. Each entry **shall** accept the optional keys `description`
(prose), `inverse` (the reverse-direction name), `sourceTypes` and `targetTypes` (lists of
element-type names; omitted = any type), `cardinality` (`N`, `N..M` or `N..*`; default `0..*`),
`acyclic` (bool, default false), `suspect` (bool, default true), `extends`, `relax` and
`coverage` (see `REQ-TRS-LINKTYPE-006`). Keys may be written camelCase or snake_case.

A link-type name and an `inverse` name **shall** match `^[a-z][A-Za-z0-9]*$` and **shall not**
collide with a built-in link or reverse-index name (`satisfies`, `verifies`, `derivedFrom`,
`refines`, `supertype`, `typedBy`, `subsets`, `redefines`, `allocatedTo`, `allocatedFrom`,
`satisfiedBy`, `verifiedBy`, `derivedChildren`, `refinedBy`, `specializedBy`, `links`, …),
with another declared type name, or with another declared `inverse`.

An entry that is structurally invalid — bad or colliding name/inverse, unparseable
`cardinality`, a non-zero lower bound without `sourceTypes`, an element-type name in
`sourceTypes`/`targetTypes` that is not a known element type, an `extends` that is not a
supported base, `relax`/`coverage` without `extends`, or a `relax` code not relaxable for the
base — **shall** raise warning `W630` naming the entry and the defect, and the entry **shall**
be ignored as a whole. An unknown key **shall** raise `W630` and be otherwise ignored.

**Acceptance criteria:** a well-formed table produces no finding; each listed defect produces
`W630`; an ignored entry is not usable by `links:` (its uses raise `E630`); a model with no
`[linkTypes]` table produces no new finding.

**Source:** `REQ-TRS-LINKTYPE-001` (product model), `ADR-SYS-LINKTYPE-001`.
