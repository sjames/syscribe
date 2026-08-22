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
solely because it contains a construct outside the mapped element set (`analysis`/`case`/
`verification def`, `calc`/`constraint`, and similar). Only a fixed set of element kinds —
`Package`, `Part(Def/Usage)`, `Attribute(Def/Usage)`, `Port(Def/Usage)`, `Connection(Def/Usage)`,
`Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`, `AllocationUsage`,
`variation`/`variant` membership, — as of `REQ-TRS-SYSMLV2-018`/`-019` —
`State(Def/Usage)`/`Action(Def/Usage)`, — as of `REQ-TRS-SYSMLV2-020`/`-021`/`-022` —
`View(Def/Usage)`, `ViewpointDef`, `ViewpointUsage`, `Rendering(Def/Usage)`, — as of
`REQ-TRS-SYSMLV2-023` — `ConcernDef`/`Concern`, — as of `REQ-TRS-SYSMLV2-024` —
`FlowDef`/`Flow`, — as of `REQ-TRS-SYSMLV2-025` — `EnumerationDef`/`Enumeration`, and — as of
`REQ-TRS-SYSMLV2-026`/`-027`/`-028` — `CaseDef`/`Case`, `AnalysisCaseDef`/`AnalysisCase`,
`VerificationCaseDef`/`VerificationCase` (but **not** `UseCaseDef`/`UseCase`, deliberately still
excluded) — are synthesized into first-class, cross-referenceable `RawElement`s. Constructs outside
that set are counted and named for browsing but not deeply modeled.

## Rationale

`sysml-v2-parser` parses the full grammar regardless of scope, so the coverage decision is about
mapping breadth, not parsing breadth. A fixed, named boundary makes "is X supported" answerable and
testable instead of an open-ended, ever-growing surface — and this particular set matches exactly
what the three cross-reference directions this feature exists to serve
(`REQ-TRS-SYSMLV2-003`/`004`/`005`) plus reasonable structural browsing actually require.

## Scope

- Full semantic mapping of calc/constraint/use-case constructs is explicitly deferred, tracked as
  follow-on scope, not required by this requirement or its siblings. `REQ-TRS-SYSMLV2-018`/`-019`
  moved State/Action, `REQ-TRS-SYSMLV2-020`/`-021`/`-022` moved View/Viewpoint/Rendering,
  `REQ-TRS-SYSMLV2-023` moved Concern, `REQ-TRS-SYSMLV2-024` moved Flow, `REQ-TRS-SYSMLV2-025` moved
  Enumeration, and `REQ-TRS-SYSMLV2-026`/`-027`/`-028` moved Case/AnalysisCase/VerificationCase
  (deliberately *not* UseCase), out of that deferred set and into the fixed mapped list above — the
  parse-broad boundary this requirement establishes never changed; only the mapped-set membership
  did, exactly as the bullet below anticipates.
- An unmapped construct is not itself an error or warning — it is simply invisible to the graph,
  the same way a native Markdown model has no way to express content that isn't frontmatter or
  documentation body.
- Growing the mapped set in a later phase is expected to extend this requirement's element list,
  not to change the parse-broad/map-narrow principle itself.
