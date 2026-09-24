---
type: Requirement
id: REQ-TRS-LINKTYPE-006
name: "A link type may extend a built-in trace link, inheriting its rules and reverse index except for explicitly relaxed codes"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-LINKTYPE-000]
breakdownAdr: Decisions::LinkTypesADR
tags:
  - link-types
---

A link type **may** declare `extends = "<base>"` where base is one of `satisfies`, `verifies`,
`derivedFrom`, `refines`. Each instance of such a type **shall** then be treated as an instance
of the base link by every rule and reverse index that applies to the base (for example `E313`
domain matching and `E312` for `satisfies`; `E104` for `verifies`; `E105`/`E310`/`W303` for
`derivedFrom`; `E316` for `refines`; and the coverage checks fed by `satisfiedBy`,
`verifiedBy`, `derivedChildren`, `refinedBy`), in addition to the type's own declared
constraints.

- `relax = [codes]` **shall** suppress the listed codes for instances of that type only.
  Relaxable codes per base: `satisfies` → `E312`, `E313`; `verifies` → `E104`;
  `derivedFrom` → `E105`, `E310`, `W303`; `refines` → `E316`. For `E310` (an element-level
  rule) the suppression applies only when every `derivedFrom`-like link on the requirement is
  of a type relaxing `E310`.
- `coverage = false` **shall** keep the base rule checks but withhold the instances from the
  base's reverse index, so they neither count toward coverage nor make their target a parent.
  Default `true`.
- The built-in fields themselves **shall** never be relaxed by any declaration.
- Mutating commands (`set`, MCP writes) **shall** never rewrite an extending link into the
  base field.

**Acceptance criteria:** an extending link triggers the base rule when not relaxed; the same
link with the code in `relax` does not; a built-in link beside it still triggers the rule; a
`coverage = true` extending `satisfies` clears `W300` on its target, a `coverage = false` one
does not.
