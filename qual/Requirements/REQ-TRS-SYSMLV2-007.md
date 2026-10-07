---
id: REQ-TRS-SYSMLV2-007
type: Requirement
name: Tool shall parse the full SysMLv2 grammar but map only a fixed set of element kinds
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** accept the full SysML v2/KerML textual grammar without failing to parse a file
solely because it contains a construct outside the mapped element set (package-level `actor`,
`filter`, KerML declarations, and similar). Only a fixed set of
element kinds — `Package`, `Part(Def/Usage)`, `Attribute(Def/Usage)`, `Port(Def/Usage)`,
`Connection(Def/Usage)`, `Interface(Def/Usage)`, `Item(Def/Usage)`, `Requirement(Def/Usage)`,
`AllocationUsage`, `AllocationDef` (as of [[REQ-TRS-SYSMLV2-029]]), and `variation`/`variant` membership — **shall** be synthesized into
first-class, cross-referenceable elements (the mapped set has since grown through
[[REQ-TRS-SYSMLV2-018]]..[[REQ-TRS-SYSMLV2-097]]; the parse-broad boundary is unchanged). A
construct outside that set **shall** be invisible to the graph — never an error, and surfaced only
by the per-file advisory `W543` count ([[REQ-TRS-SYSMLV2-030]]) — the same way a native Markdown
model has no way to express content that isn't frontmatter or documentation body.

**Source:** `REQ-TRS-SYSMLV2-007` (product model).

**Acceptance criteria:** a single `.sysml` file mixing a mapped construct (e.g. a `part def`) and
an unmapped one (e.g. a package-level `actor`) parses with no error; the mapped construct appears
as a first-class element under its derived qualified name; the unmapped construct contributes no
element and no `Finding` names it (only the `W543` count).
