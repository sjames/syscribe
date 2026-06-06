---
id: REQ-TRS-MOVE-004
type: Requirement
name: Move Preserves Stable Identifiers
title: Tool shall preserve stable IDs and references made through them when moving
status: draft
reqDomain: software
verificationMethod: test
---

A move changes only the path-derived qualified name of an element. The tool **shall not** change an element's stable identifier (`REQ-*`, `TC-*`, `ADR-*`, and the other opaque id schemes) when moving it, and **shall not** alter references expressed through those stable identifiers (e.g. `verifies:`/`derivedFrom:` entries given as `REQ-*` ids), since such references are location-independent and remain valid after the move.

**Source:** ID Scheme (§ID); Feature request.

**Acceptance criteria:** after moving a native `Requirement`/`TestCase`, its `id` is unchanged and any `verifies:`/`derivedFrom:` references that used its stable id still resolve, without those entries being rewritten.
