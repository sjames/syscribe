---
id: REQ-TRS-DIAG-003
type: Requirement
name: Tool shall check SVG id consistency (W406/W407) only on diagrams whose SVG is inline
status: draft
reqDomain: software
verificationMethod: test
---

The SVG id-consistency rules `W406` (a `shapes:`/`edges:` id with no matching
`id` attribute in the SVG) and `W407` (an SVG `id` with no manifest entry) are
scoped by §8.16.7 step 3 to **inline SVG**. The tool **shall** apply them only
to a `Diagram` whose rendering path carries its SVG inline in the body, and
**shall not** apply them to a diagram that has no inline SVG by design:

- a **PlantUML companion** diagram (`pumlMode: companion`) — its `shapes:` /
  `edges:` manifest feeds the generated `.puml`, and the `.puml`/`.svg`
  companion files are the source of truth;
- a **companion SVG** diagram (`svgMode: companion`, or `svgFile:` set);
- a **Mermaid** or **inline PlantUML** diagram (`diagramKind: Mermaid` /
  `PlantUML`), whose body is a ` ```mermaid ` / ` ```plantuml ` block;
- a **structured SVG** diagram (a `layout:` block) whose body has no
  ` ```svg ` block — the SVG is built server-side from the manifest.

A hand-authored inline-SVG diagram (`svgMode: inline`, or `svgMode:` absent on
a diagram not covered above) **shall** still raise `W406`/`W407` for every
unmatched id, exactly as before.

**Source:** GH issue #158; spec §8.16.1, §8.16.7 step 3.

**Acceptance criteria:** a `pumlMode: companion` diagram with a `shapes:`/
`edges:` manifest and an image reference to its anticipated `.svg` raises no
`W406`/`W407`; a structured diagram with `layout:` and no ` ```svg ` block
raises no `W406`; an inline-SVG diagram whose SVG lacks a manifest id still
raises `W406`.
