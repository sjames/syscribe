---
type: Requirement
id: REQ-TRS-MCP-003
name: "Token-efficient read tools for search, retrieval, navigation, trace, and validation"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall expose a curated set of read tools for navigating and validating the
model, designed for token efficiency: `search`, `get_element`, `list_by_type`, `tree`,
`neighbors`, `graph_query`, `trace`, `impact`, `validate`, and `validate_element`.

## Token-efficiency requirements

- **Summaries by default.** Element-returning tools shall return a compact summary
  (`qname`, `id`, `type`, `name`, and minimal status/excerpt) unless the caller opts into a
  fuller view via a `detail` flag and/or a `fields` projection that limits the returned keys.
- **Chainable identifiers.** Every returned element shall carry both its `qname` and its
  stable `id` (when present) so the client can chain follow-up calls without re-searching.
- **Bounded lists.** Every list-shaped result shall support `limit`/`offset` paging and report
  the total count, so the client controls payload size.
- **Reference flexibility.** Any element-reference parameter shall accept a stable id, a
  qualified name, or a display name, resolved with the same precedence as the CLI.
- **Generic graph traversal with a one-hop convenience.** A single `graph_query` tool shall
  serve multi-hop connectivity and path queries across selectable edge kinds and traversal
  directions, rather than a separate tool per relationship type. The `neighbors` tool is a
  bounded one-hop convenience over the same edge model (the immediate adjacencies of a single
  element); it shall not be implemented as a family of per-edge tools.
- **Two validation scopes.** `validate` shall return the findings for the whole model
  (optionally filtered by severity), whereas `validate_element` shall return only the findings
  attributable to a single referenced element, so a client can check one element without
  paying for the full-model report.

Results shall be returned as structured JSON, not human-formatted Markdown.
