---
id: REQ-TRS-SYSMLV2-007
type: Requirement
name: Tool shall parse the full SysMLv2 grammar but map only a fixed set of element kinds
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** accept the full SysML v2/KerML textual grammar without failing to parse a file
solely because it contains a construct outside the mapped element set (behavior bodies,
`analysis`/`case`/`verification def`, `calc`/`constraint`, and similar). Only a fixed set of
element kinds — `Package`, `Part(Def/Usage)`, `Attribute(Def/Usage)`, `Port(Def/Usage)`,
`Connection(Def/Usage)`, `Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`,
`AllocationUsage`, and `variation`/`variant` membership — **shall** be synthesized into
first-class, cross-referenceable elements. A construct outside that set **shall** be invisible to
the graph — not an error, not a warning — the same way a native Markdown model has no way to
express content that isn't frontmatter or documentation body.

**Source:** `REQ-TRS-SYSMLV2-007` (product model).

**Acceptance criteria:** a single `.sysml` file mixing a mapped construct (e.g. a `part def`) and
an unmapped one (e.g. a `state def`) parses with no error; the mapped construct appears as a
first-class element under its derived qualified name; the unmapped construct contributes no
element and no `Finding` at all.
