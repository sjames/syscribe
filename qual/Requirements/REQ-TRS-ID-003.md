---
id: REQ-TRS-ID-003
type: Requirement
name: ADR ID Pattern
title: Tool shall validate that ADR elements carry an ADR-* id
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** validate that every element with `type: ADR` carries an `id:` field matching the pattern `^ADR(-[A-Z0-9]{2,12})+-[0-9]{3,8}$`. Missing `id:` shall produce `E301`; present but non-matching `id:` shall produce `E300`.

**Source:** §11.12 `E300`, `E301`

**Acceptance criteria:** `id: ADR-SYS-001` passes; `id: ADR-sys-001` produces `E300`; absent `id:` produces `E301`.
