---
type: Requirement
id: REQ-TRS-SYSMLV2-007
name: "Full-grammar parsing, fixed-set element mapping: an unmapped SysMLv2 construct never breaks ingestion"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

Syscribe shall accept the full SysML v2/KerML textual grammar without failing to parse a file
solely because it contains a construct outside the mapped element set (behavior bodies, `analysis`/
`case`/`verification def`, `calc`/`constraint`, and similar). Only a fixed set of element kinds —
`Package`, `Part(Def/Usage)`, `Attribute(Def/Usage)`, `Port(Def/Usage)`, `Connection(Def/Usage)`,
`Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`, `AllocationUsage`, and
`variation`/`variant` membership — are synthesized into first-class, cross-referenceable
`RawElement`s. Constructs outside that set are counted and named for browsing but not deeply
modeled.

## Rationale

`sysml-v2-parser` parses the full grammar regardless of scope, so the coverage decision is about
mapping breadth, not parsing breadth. A fixed, named boundary makes "is X supported" answerable and
testable instead of an open-ended, ever-growing surface — and this particular set matches exactly
what the three cross-reference directions this feature exists to serve
(`REQ-TRS-SYSMLV2-003`/`004`/`005`) plus reasonable structural browsing actually require.

## Scope

- Full semantic mapping of behavior/analysis/case/calc/constraint constructs is explicitly
  deferred, tracked as follow-on scope, not required by this requirement or its siblings.
- An unmapped construct is not itself an error or warning — it is simply invisible to the graph,
  the same way a native Markdown model has no way to express content that isn't frontmatter or
  documentation body.
- Growing the mapped set in a later phase is expected to extend this requirement's element list,
  not to change the parse-broad/map-narrow principle itself.
