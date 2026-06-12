---
id: REQ-TRS-XREF-003
type: Requirement
name: Tool shall emit a reference error without aborting on unresolved references
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit a reference error finding for any cross-reference that cannot be resolved after full model loading. An unresolved reference **shall not** cause a fatal error or halt validation of other elements.

**Source:** §11.5 ¶4

**Acceptance criteria:** A model with one dangling `supertype:` reference produces a reference error for that element and correct findings for all other elements.
