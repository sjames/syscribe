---
type: Requirement
id: REQ-TRS-SUS-LINKS-001
name: "traceBaselines frontmatter field stores per-target content hashes"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-SUS-LINKS-000]
breakdownAdr: Decisions::SuspectLinksADR
tags:
  - traceability
  - suspect-links
---

The **source** element — the element that holds the trace link, per the upstream link
direction of §12.1 — shall carry an optional frontmatter field `traceBaselines:`. It is a
mapping from a **target identifier** to an **algorithm-prefixed content hash** of that
target's projection (REQ-TRS-SUS-LINKS-002).

- The **key** is the target identifier exactly as authored on the link — a stable id
  (e.g. `REQ-SCHED-BITMAP-001`) or a qualified name — resolvable by the standard resolver.
- The **value** is a string of the form `<algo>:<hex>` (e.g. `blake3:9f2a3c1d…`).
- For a multi-valued link (e.g. `verifies: [A, B]`), **each** referenced target has its
  own entry; one `traceBaselines` map covers all of a source's links regardless of kind.
- The field is **optional**; its absence means the source has no baselined links.
- Serialization shall be deterministic (sorted keys), consistent with the existing
  canonical-output conventions.

Example:

```yaml
verifies: [REQ-SCHED-BITMAP-001]
traceBaselines:
  REQ-SCHED-BITMAP-001: "blake3:9f2a3c1d"
```

The field shall **not** record `acceptedAt` / `acceptedBy` or other provenance: the hash
is the sole persisted state, and the identity/time of a re-baseline is recovered from
version-control history.
