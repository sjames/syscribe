---
type: Requirement
id: REQ-TRS-MCP-026
name: "search supports server-side filters and full-text body matching"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - retrieval
---

The `search` tool shall support server-side filtering and full-text matching, so an LLM can
retrieve precisely without over-fetching and post-filtering.

## Filtering and matching

- `search` shall accept optional `type`, `status`, and `domain` filters, and an optional
  `where` custom-field predicate using the same syntax as the CLI `--where`
  (`custom.<key><op><value>`).
- Matching shall consider element documentation **body** text in addition to the name, stable
  id, and qualified name.
- Filters compose with logical AND; when one or more filters are supplied the `query` term
  becomes optional (filters alone may select results).
- The result shape (`{results:[{qname,id,type,name,score}], total}`) and `limit`/`offset`
  paging of REQ-TRS-MCP-003 are unchanged.
