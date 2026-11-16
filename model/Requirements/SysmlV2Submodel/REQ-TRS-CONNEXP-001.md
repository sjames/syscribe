---
type: Requirement
id: REQ-TRS-CONNEXP-001
name: "A connection keeps its own endpoints through SysMLv2 ingest and export, and export --ndjson gives its qualified name, kind and endpoints together"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - connections
  - export
---

SysMLv2 ingestion shall record each named `connection` usage's name on the `connections:` entry it
lifts onto the owning part, and `export-sysml` shall pair a child `Connection` element with its
owner's entry by that name — never by position or by `typedBy`, which several connections share.

`export` (and `export --ndjson`) shall add to the record of every `Connection`/`Interface` that has
resolvable endpoints `source` and `target` (the first two ends) and `ends` (every end, with its
`end` role when named), beside the `qname` and the `typedBy` already in its frontmatter.

## Rationale

GH #486: the twin of a lifted entry was found by type when the entry had no name, so three
connections of one type took each other's endpoints once their names sorted differently from the
source order. A tool generating rules from typed connections also had no single export joining a
connection's kind to its endpoints.
