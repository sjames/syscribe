---
type: Requirement
id: REQ-TRS-MCPGUIDE-001
name: "the MCP server ships model-specific instructions and workflow prompts"
status: draft
reqDomain: software
reqClass: system
tags:
  - mcp
  - agents
---

The MCP server shall tell a client how to use it safely against *this* model, and offer ready-made workflows with the exact tool sequence (GH #440).

## Behavior

- The `initialize` instructions are generated from the loaded model and `.syscribe.toml`: tool order for common jobs, the safe-mutation rules (never invent an id; never set `claimedBy`/`claimedAt` through a tool), the project's id prefixes including `[ids.prefixes]` extras, the status vocabulary of each lifecycle type, and the configured gates and profiles. A `--read-only` server states that it is read-only and advertises no write sequence. If a reload holds the model the model-independent part is still returned.
- `syscribe mcp --print-instructions` prints the same text.
- Prompts `add-requirement-with-test`, `record-test-evidence`, `review-change-set`, `close-planning-item` and `check-variant`, each with named arguments; a missing required argument is an error. Every tool a prompt names exists, and the sequences run on the example models.
- `review-change-set` relies on two read tools that are part of this behaviour: `changes {from, to?, types?, package?}` and `validate {new_since}` (only findings absent at that revision, with a `preexisting` count).
