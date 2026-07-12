---
type: Requirement
id: REQ-TRS-BL-002
name: "Full canonical content seal over the in-scope graph"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-BL-000]
breakdownAdr: Decisions::BaselineADR
tags:
  - baseline
  - hashing
---

The baseline **seal** shall be a content hash that freezes the in-scope elements
byte-exactly.

## Per-element hash

- For each in-scope element, a **BLAKE3** hash shall be computed over its **full canonical
  content**: all frontmatter *and* the markdown body, using the same canonicalization as
  suspect-link projection (sorted keys, null-stripping, line-ending normalization). This
  shall be implemented by refactoring the suspect `projection_hash` to accept an exclusion
  set rather than duplicating the canonicalization.
- Unlike the suspect projection, **no normative/editorial distinction is drawn**: editorial
  fields (`name`, `extRef`, `displayOrder`) are included, so any in-scope edit — cosmetic or
  normative — changes the element hash. The only intrinsic exclusion is a `Baseline`'s own
  `seal` block, which cannot hash itself.

## Aggregate seal

- The per-element hashes shall be aggregated deterministically: the elements shall be sorted
  by stable id (falling back to qualified name), each contributing its `id` (or qname) and
  its per-element hash, and the aggregate computed by hashing that ordered, delimited
  encoding — so the result is independent of file ordering, map-iteration order, and
  platform locale.
- The aggregate shall be stored algorithm-prefixed (`blake3:<hex>`) in the element's `seal`
  block together with the in-scope `elementCount`, and shall reproduce exactly for unchanged
  in-scope content.

## Baselines excluded from scope

- `Baseline` elements shall themselves be excluded from scope resolution and therefore from
  every seal, so that sealing one baseline never perturbs another's hash (REQ-TRS-BL-003).

The content hash is a portable, version-control-agnostic proof; the captured `gitCommit`
(REQ-TRS-BL-004) is the authoritative anchor to the sealed source state.
