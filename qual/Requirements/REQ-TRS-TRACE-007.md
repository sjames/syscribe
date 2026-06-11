---
id: REQ-TRS-TRACE-007
type: Requirement
title: "Tool shall emit E315 for cross-domain supertype: or typedBy: links"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E315` when an element with `domain: software` has a `supertype:` or `typedBy:` reference that resolves to an element with `domain: hardware`, or vice versa.

**Source:** §12.6; §11.12 `E315`

**Acceptance criteria:** A `PartDef` with `domain: software` and `supertype:` pointing to a `PartDef` with `domain: hardware` produces `E315`. Cross-domain integration via an explicit `Allocation` element does not produce `E315`.
