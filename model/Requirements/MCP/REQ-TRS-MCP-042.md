---
type: Requirement
id: REQ-TRS-MCP-042
name: "generate_view synthesises diagram source from the model graph"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPEvidenceToolsADR
tags:
  - mcp
---

The MCP server shall expose a `generate_view` read tool that synthesises diagram source (Mermaid by
default) from the model graph for a requested view kind, without requiring hand-maintained `Diagram`
elements to exist.

## Behaviour

- Required kinds: `traceability` (Requirement → TestCase → source), `containment`
  (package/namespace tree), `feature` (the feature model), `allocation` (source → target).
- Inputs: `{ kind, root?, depth?, format? }`; `root` scopes the view, `depth` bounds traversal,
  `format` selects the diagram language.
- The view shall be derived solely from the typed model graph (as exposed via `graph_query` /
  `neighbors`), and the generated source shall embed resolvable `%% ref:` / `sysml:ref` annotations
  so it round-trips through `lint_docs` / the diagram checks with zero new findings.
- The tool is read-only and returns source only; it does not persist a `Diagram` element.
