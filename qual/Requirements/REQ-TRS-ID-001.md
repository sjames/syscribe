---
id: REQ-TRS-ID-001
type: Requirement
name: Requirement ID Pattern
title: Tool shall validate that Requirement elements carry a REQ-* id
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** validate that every element with `type: Requirement` carries an `id:` field matching the pattern `^REQ(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`. Failure to match **shall** produce error `E006`.

**Source:** §11.12 `E006`; CLAUDE.md §ID Scheme

**Acceptance criteria:** A `Requirement` with `id: REQ-TRS-001` passes; `id: REQ-trs-001` (lowercase) and `id: REQTRS001` (no hyphens) each produce `E006`.
