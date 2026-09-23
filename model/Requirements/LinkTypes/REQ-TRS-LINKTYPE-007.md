---
type: Requirement
id: REQ-TRS-LINKTYPE-007
name: "A follow command shall traverse a named link type forward or in reverse, one hop or transitively"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

The CLI **shall** provide `syscribe -m <root> follow <elem> <link> [--reverse] [--transitive]
[--depth N] [--format text|json|dot]`. `<elem>` is an id or qualified name. `<link>` is a
declared link-type name (forward), its declared `inverse` (reverse), or a built-in link name
(`satisfies`, `verifies`, `derivedFrom`, `refines`, `supertype`, `typedBy`, `allocatedTo`) or
built-in reverse name (`satisfiedBy`, `verifiedBy`, `derivedChildren`, `refinedBy`,
`specializedBy`, `allocatedFrom`). `--reverse` flips the direction. By default one hop is
followed; `--transitive` follows to a fixed point; `--depth N` bounds the hops (and implies
transitive). Each reached element is reported once with its hop depth, id/qname, type and name.
`json` emits `{start, link, direction, results:[{qname,id,type,name,depth,from}]}`; `dot`
emits a Graphviz digraph. An unknown element or link name **shall** exit non-zero, and an
unknown link name **shall** print the available link names.

**Acceptance criteria:** forward, inverse and `--reverse` traversal return the expected
elements; transitive traversal reaches multi-hop elements and terminates on cycles; `--depth`
bounds it; json is valid and complete; an unknown link exits non-zero listing the names.
