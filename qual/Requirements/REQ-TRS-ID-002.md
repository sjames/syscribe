---
id: REQ-TRS-ID-002
type: Requirement
title: Tool shall validate that TestCase elements carry a TC-* id
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** validate that every element with `type: TestCase` carries an `id:` field matching the pattern `^TC(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`. Failure to match **shall** produce error `E006`.

**Source:** §11.12 `E006`; CLAUDE.md §ID Scheme

**Acceptance criteria:** A `TestCase` with `id: TC-TRS-PARSE-001` passes; `id: TC-001` (too short) produces `E006`.
