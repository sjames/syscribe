---
type: Requirement
id: REQ-TRS-MCP-002
name: "Model is loaded once into a shared store with an explicit reload tool"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
---

The MCP server shall load the model once at startup into an in-memory store (parsed elements,
graph, resolver, and validation config) shared across all tool calls, and shall expose a
`reload` tool that re-walks the model from disk and rebuilds the store on demand.

## Behaviour

- A tool call reads from the cached store rather than re-walking the model, so a sequence of
  read calls sees a consistent snapshot.
- The `reload` tool re-runs the load (walk → derive → graph → resolver) and replaces the store
  contents, returning the post-reload element count so the client can confirm the refresh.
- After any successful write (REQ-TRS-MCP-005..007) the server shall rebuild the store
  automatically so subsequent reads reflect the change without an explicit `reload`.

Edits made outside the server are picked up automatically by the file watcher
(REQ-TRS-MCP-048, GH #181); the explicit `reload` tool remains as a way to force a re-read, and is
the recovery path when the server runs with `--no-watch`.
