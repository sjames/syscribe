---
type: Requirement
id: REQ-TRS-MCP-040
name: "render_diagram returns diagram source plus structural findings (no image rendering)"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `render_diagram` read tool that, given a `Diagram` element reference,
returns the diagram's **source** together with its structural-check findings. It shall **not**
render the diagram to an image; rendering is a separate concern outside syscribe.

## Behaviour

- The result shall be `{ format, source, findings }`. `source` is the PlantUML source generated from
  the `Diagram` element (default), or the Mermaid source for a Mermaid diagram.
- `findings` shall be the diagram's `W400`–`W415` structural checks, so an invalid-but-describable
  diagram is still flagged.
- The tool shall not shell out to or bundle any external renderer; it produces no SVG/PNG artifact.
