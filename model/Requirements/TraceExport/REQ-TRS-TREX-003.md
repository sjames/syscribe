---
type: Requirement
id: REQ-TRS-TREX-003
name: "The export accepts a sort order — directory (default), ascending or descending by qualified name — applied to every list"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-TREX-000]
breakdownAdr: Decisions::TraceExportADR
tags:
  - traceability
  - export
---

`--sort directory|asc|desc` shall order the top-level `requirements` list and every nested reference list: `directory` (the default) is the walker's file order (the order `export` and `ls` use); `asc` and `desc` are lexicographic by full qualified name. The chosen order is recorded in the document's `sort` field. An unknown value is a usage error naming the three valid values, exit 1. For a given model, option set and sort the output shall be byte-identical across runs.

## Rationale

A declared, deterministic order makes two exports diffable and lets a reader choose the model's own structure or an alphabetical index.

## Scope

- Sorting is by qualified name only; sorting by id or status is not provided.
