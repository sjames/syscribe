---
type: Requirement
id: REQ-TRS-TREX-001
name: "The trace document lists every requirement with identity, derivation, satisfaction, verification and a computed coverage block, every reference by full qualified name"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-TREX-000]
breakdownAdr: Decisions::TraceExportADR
tags:
  - traceability
  - export
---

The export shall emit one JSON object `{ "version": 1, "modelRoot", "config": null | {...}, "sort", "requirements": [...], "summary": {...} }`. Each entry of `requirements` shall carry: `qname` (full qualified name), `id` (stable id, or null), `name`, `type` (`Requirement`, `RequirementDef` or SysML `Requirement`), `status`, `reqClass`, `reqDomain`, `file` (model-root-relative path); `derivedFrom` and `derivedChildren` (lists of `{qname, id}`), `breakdownAdr` (`{qname, id, status}` or null); `satisfiedBy` (list of `{qname, id, type, domain}`), `verifiedBy` (list of `{qname, id, testLevel, status, verdict}` where `verdict` is the ingested result — `pass`/`fail`/`unknown` — or null without a results sidecar), `refinedBy` (list of `{qname, id}`); and `coverage`: `{ "leaf": bool, "satisfied": bool, "verified": bool, "integrationVerified": bool }` computed by the same rules `W300`, `W002` and `W305` apply (a leaf is satisfied when at least one satisfier exists; verified when at least one active TestCase verifies it; a parent is integration-verified when an active L3/L4/L5 TestCase verifies it). Custom link types that `extends` a built-in link with `coverage = true` contribute to the same lists. `summary` shall give the counts of requirements, leaves, satisfied, verified and integration-verified. Every reference shall be the element's full qualified name; a dangling reference (one that does not resolve) shall appear as `{ "qname": "<text as written>", "unresolved": true }`, never dropped.

## Rationale

The lists are the reverse indices `validate` already maintains; exposing them with the same coverage semantics keeps an external checker and the validator in agreement.

## Scope

- Native `Requirement`s and SysML `RequirementDef`/`Requirement` (including SysMLv2-ingested ones) are all included; `TestCase`s with `status: retired` are listed but do not count as verification.
- Field order within objects is fixed as listed so the document diffs cleanly.
