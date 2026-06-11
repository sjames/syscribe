---
id: REQ-TRS-TRACE-001
type: Requirement
title: Tool shall enforce that traceability links point upstream
status: draft
reqDomain: software
verificationMethod: analysis
---

The tool **shall** enforce the OSLC link direction convention: the derived, verifying, or satisfying artifact holds the traceability reference, not the upstream artifact. Specifically, `derivedFrom:`, `verifies:`, and `satisfies:` fields are owned by the downstream element.

**Source:** §12.1

**Rationale:** Reversing link direction would produce incorrect computed reverse indices (`derivedChildren`, `verifiedBy`) and mislead coverage analysis.

**Acceptance criteria:** The computed reverse indices (`verifiedBy`, `derivedChildren`) are populated correctly from the `verifies:` and `derivedFrom:` fields of downstream elements.
