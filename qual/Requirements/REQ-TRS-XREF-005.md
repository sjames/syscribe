---
id: REQ-TRS-XREF-005
type: Requirement
title: "Tool shall resolve verifies: and derivedFrom: references by stable ID"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** resolve references in `verifies:`, `derivedFrom:`, `breakdownAdr:`, and `derivedFromSafetyGoal:` fields by matching the referenced string against the `id:` field of all loaded elements, in addition to qualified name matching.

**Source:** §11.10

**Acceptance criteria:** A `verifies: REQ-TRS-PARSE-001` reference resolves to the element whose `id:` is `REQ-TRS-PARSE-001` regardless of its file path.
