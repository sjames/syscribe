---
id: REQ-TRS-ELEM-002
type: Requirement
title: "Tool shall emit E005 for unrecognised type: values"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit error `E005` for any element whose `type:` value is not in the element type inventory (§2), and **shall** skip that element from further processing.

**Source:** §11.12 `E005`

**Acceptance criteria:** A file with `type: BogusWidget` produces exactly one `E005` finding and no other type-related errors.
