---
id: REQ-TRS-SYSMLV2-015
type: Requirement
name: A genuinely two-segment connect endpoint that falls back to head-only raises W542 identifying the dropped segment
status: draft
reqDomain: software
verificationMethod: test
---

A `connect` endpoint whose genuinely two-segment dotted chain falls back to head-only
qualification (the tail isn't a locally-redeclared feature) **shall** raise a `W542` finding
identifying the endpoint text and the head-only edge it was truncated to. A chain that resolves
via redeclaration, a bare endpoint, and a three-or-more-segment chain **shall** all raise no
`W542`.

**Source:** `REQ-TRS-SYSMLV2-015` (product model), `ADR-SYS-SYSMLV2-001` addendum.
