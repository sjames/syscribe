---
type: Requirement
id: REQ-TRS-SYSMLV2-004
name: "A native TestCase's verifies: field can target a SysMLv2 element"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
  - traceability
---

A native Syscribe `TestCase`'s existing `verifies:` field shall also resolve against the qname
index of any ingested SysMLv2 subtree, so a `TestCase` can verify a SysMLv2-authored element (a
`PartUsage`, `ActionUsage`, etc.) the same way it verifies a native `Requirement` today.

## Rationale

One field, one mental model — "what does this test verify" — regardless of which side of the
format boundary the target lives on. A separate, dedicated field for this would fragment
verification semantics across two differently-named fields for no conceptual gain, since a
`TestCase` verifying "a requirement" and verifying "a SysMLv2 part or behavior" are the same kind
of claim from the test's point of view.

## Scope

- No change to `TestCase`'s schema or its existing validation rules beyond widening what
  `verifies:` is allowed to resolve to.
- Computed reverse indices (`verifiedBy`, `CLAUDE.md` §11.11) include SysMLv2-element targets
  exactly like `Requirement` targets — no separate index is introduced.
- This requirement covers `TestCase` → SysMLv2 element only; a SysMLv2 element's own `satisfy`/
  `verify` targeting a `Requirement` is the other direction, `REQ-TRS-SYSMLV2-003`.
