---
type: Requirement
id: REQ-TRS-LSP-006
name: "Workspace symbol search finds elements by name, id, or qualified name"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-LSP-000]
breakdownAdr: Decisions::LSPServerADR
tags:
  - lsp
  - navigation
---

The server shall implement `workspace/symbol`. Given a query string, it shall return matching
elements from across the whole model — matched against `name`, `id` (if id-identified), and
qualified name — each as a `SymbolInformation`/`WorkspaceSymbol` with its `kind` set from the
element's `type` and its `location` set to the element's defining file.

## Behavior

- Matching reuses the existing search behavior (`ftsearch`/`query` modules) rather than a new
  index, so results stay consistent with the equivalent CLI/MCP search tools.
- An empty or non-matching query returns an empty result, not an error.
- Results are capped to a bounded count (matching existing CLI/MCP search result limits) to
  keep the response size predictable on large models.
