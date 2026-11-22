---
type: ReviewRecord
id: RR-TRS-001
name: "Owner approval of the scope changes made while fixing GH #513 and #516"
status: closed
reviewType: requirements_review
reviewDate: "2026-11-22"
reviews:
  - REQ-TRS-ACTSEND-001
  - REQ-TRS-SYSCOMP-001
---

The owner reviewed and approved two changes to approved requirements, in the working session of 2026-11-22.

- **REQ-TRS-ACTSEND-001 (GH #516).** The owner chose correctness over backward compatibility: a `SendAction` makes an `ActionDef` executable, and the earlier `variables: []` opt-in is dropped.
- **REQ-TRS-SYSCOMP-001 (GH #513).** The owner approved the proposed rule: in a composite, a plain `accept: X` receives a port message when the part has exactly one in port; with several in ports the accept must name one with `via:`, and `W936` says so.

This record is filed at the owner's instruction so the two `scope-change` amendments carry an owner reference (`W325`).
