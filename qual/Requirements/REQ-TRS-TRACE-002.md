---
id: REQ-TRS-TRACE-002
type: Requirement
title: "Tool shall emit E310 for a derived Requirement with no breakdownAdr:"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E310` when a `Requirement` element has one or more `derivedFrom:` entries but does not set a `breakdownAdr:` field referencing an accepted ADR.

**Source:** §12.2; §11.12 `E310`

**Acceptance criteria:** A `Requirement` with `derivedFrom: [REQ-PARENT-001]` and no `breakdownAdr:` produces exactly one `E310` finding.
