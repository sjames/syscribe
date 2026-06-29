# mcp — run an MCP server over stdio for LLM clients

`syscribe mcp -m <model>` starts a Model Context Protocol (MCP) server that
speaks newline-delimited JSON-RPC 2.0 over **stdio**. It lets an MCP-capable LLM
client query and guard-write the Syscribe model bound at `-m`.

## Usage

    syscribe -m <model> mcp

The server runs until its stdin is closed. It is intended to be spawned by an
MCP client (an editor, an agent runtime, …), not invoked interactively.

## Read tools

- `get_element {ref, detail?, fields?}` — fetch one element by stable id,
  qualified name or display name. Summary by default; `detail:true` adds the
  Markdown body and the full frontmatter.
- `search {query, type?, limit?, offset?}` — ranked element search.
- `list_by_type {type, limit?, offset?}` — every element of a type.
- `tree {ref?, depth?}` — containment subtree.
- `neighbors {ref, edges?, direction?}` — adjacent graph nodes.
- `graph_query {from, to?, edges?, direction?, depth?}` — a typed-edge graph walk.
- `validate {file?, severity?, limit?}` — validation findings.
- `validate_element {ref}` — findings scoped to one element's file.
- `reload {}` — re-read the model from disk.

## Guarded-write tools

All write tools default to `dry_run: true`: they report the validation delta of
the proposed change without touching disk. Pass `dry_run: false` to commit. A
commit that would introduce a *new* validation error is refused
(`written: false`) unless `SYSCRIBE_MCP_ALLOW_NEW_ERRORS=1` is set.

- `create_element {qname, type, fields?, doc?, dry_run?}`
- `update_element {ref, fields?, doc?, dry_run?}`
- `move_element {ref, dest, dry_run?}`

## Resources & prompts

The format specification is exposed as `syscribe://spec/<section>` resources,
and the model-authoring prompt is exposed as the `create-model` prompt.
