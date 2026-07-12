---
type: Requirement
id: REQ-TRS-SUS-LINKS-002
name: "Content projection and BLAKE3 hashing of trace-link targets"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
---

The baseline hash shall be computed over a canonical **projection** of the target
element, not over the whole file.

## Projection surface (v1 default)

A single default projection shall apply to all element types:

- **Included:** the markdown body, plus the normative frontmatter fields (for example
  `status`, `reqDomain`, safety fields such as SIL/ASIL, `multiplicity`, and the
  type-load-bearing structural fields such as `supertype` / `typedBy`).
- **Excluded:** editorial and presentation fields that do not affect the meaning a
  downstream consumer depends on — `displayOrder`, `extRef`, `name`, layout coordinates,
  and comments — and the `traceBaselines` field itself.

Per-type projection surfaces may be refined in a later phase without changing the storage
format of REQ-TRS-SUS-LINKS-001.

## Canonicalization and digest

- Included frontmatter shall be serialized in a canonical form — sorted keys and
  null-stripping, consistent with the existing canonical serialization — and line endings
  normalized, so that cosmetic reformatting does not change the hash.
- The digest algorithm shall be **BLAKE3**.
- The stored value shall be algorithm-prefixed (`blake3:<hex>`), so the algorithm — and,
  if introduced, a projection-version marker — can be migrated unambiguously.

Two elements whose projections are byte-identical after canonicalization shall produce
identical hashes; a change confined entirely to excluded fields shall **not** change the
hash.
