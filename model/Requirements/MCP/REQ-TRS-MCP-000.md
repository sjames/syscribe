---
type: Requirement
id: REQ-TRS-MCP-000
name: "LLM clients can communicate with the model over MCP"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - mcp
  - llm
---

Syscribe shall provide a Model Context Protocol (MCP) interface so that LLM clients can
query and author the model through structured, typed operations, without parsing
human-formatted CLI output or re-deriving model structure from raw files.

## Rationale

Syscribe is built for LLM-assisted authoring, yet an LLM can reach the model today only by
reading/grepping `.md` files (forcing it to re-derive qualified names, the trace graph, and
cross-reference resolution the tool already computes) or by shelling ~60 CLI commands and
parsing Markdown sized for humans. Both waste tokens and provide no validation feedback loop
while editing.

An MCP interface closes this gap: the tool exposes the model as a small set of structured
operations the client calls directly, returning compact JSON and validation results. This
makes the single-source-of-truth model directly and efficiently consumable by the agents that
increasingly author it.
