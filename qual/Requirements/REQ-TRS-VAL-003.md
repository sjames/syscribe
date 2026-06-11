---
id: REQ-TRS-VAL-003
type: Requirement
title: Tool shall enforce warning rules W001–W007 and W300–W305
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every warning rule in the following table, as defined in §11.12.

| Code | Condition |
|---|---|
| `W001` | `Requirement` normative text contains no `shall` |
| `W002` | Approved/implemented `Requirement` has no active `TestCase` in `verifiedBy` |
| `W003` | Verified `Requirement` with empty or all-retired `verifiedBy` |
| `W004` | `sourceFile:` path does not exist on disk |
| `W005` | `Requirement` has neither `derivedFrom:` nor `derivedChildren` |
| `W006` | Both `silLevel:` and `asilLevel:` set on the same element |
| `W007` | Frontmatter contains an unrecognised key (lenient mode) |
| `W300` | Leaf `Requirement` at approved/implemented with no satisfying element |
| `W301` | Leaf `Requirement` satisfied by more than one element |
| `W302` | Leaf `Requirement` at implemented/verified still has `reqDomain: system` |
| `W303` | `breakdownAdr:` references a proposed ADR but requirement is approved or higher |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` |
| `W305` | Parent `Requirement` at approved/implemented/verified has no system-level `TestCase` |

**Source:** §11.12 (warnings), §12

**Acceptance criteria:** For each warning code, a crafted model triggering that condition produces a finding with that code and severity `Warning`.
