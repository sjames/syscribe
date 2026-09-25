---
id: REQ-TRS-VAL-003
type: Requirement
name: Tool shall enforce warning rules W001–W007 and W300–W305 (W301 retired)
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
| `W005` | `Requirement` has no upstream link (`derivedFrom:`, `derivedFromSafetyGoal:` or `derivedFromCybersecurityGoal:` — see REQ-TRS-TRACE-011) and no `derivedChildren` |
| `W006` | Both `silLevel:` and `asilLevel:` set on the same element |
| `W007` | Frontmatter contains an unrecognised key (lenient mode) |
| `W300` | Leaf `Requirement` at approved/implemented with no satisfying element |
| `W302` | Leaf `Requirement` at implemented/verified still has `reqDomain: system` |
| `W303` | `breakdownAdr:` references a proposed ADR but requirement is approved or higher |
| `W304` | `isDeploymentPackage: true` combined with `domain: hardware` |
| `W305` | Parent `Requirement` at approved/implemented/verified has no system-level `TestCase` |

`W301` (leaf `Requirement` satisfied by more than one element) is **retired** (GH #121): a leaf
`Requirement` satisfied by more than one element — structural, behavioural or a mix — is
legitimate and the tool **shall not** raise any finding for the satisfier count alone. Whether a
requirement is a leaf or a parent **shall** depend only on `derivedFrom` (`derivedChildren`);
`E312` (parent requirement in a `satisfies:` list) and `W300` (leaf with no satisfier) are
unchanged.

**Source:** §11.12 (warnings), §12.3

**Acceptance criteria:** For each warning code, a crafted model triggering that condition produces a finding with that code and severity `Warning`. A leaf requirement satisfied by two
elements (e.g. a `PartDef` and a `StateDef`) produces no `W301` and no other satisfier-count finding.
