---
type: Requirement
id: REQ-TRS-MCP-048
name: "The MCP server reloads the model automatically when its input files change on disk"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-MCP-000]
breakdownAdr: Decisions::MCPServerADR
tags:
  - mcp
  - reload
---

While serving, `syscribe mcp` shall watch the files its store is built from and reload the
store automatically when they change outside the server, so a long-lived LLM client never
answers from a stale model after an editor save, a `git checkout`, or another tool's write.

## Behaviour

- **What is watched.** The model root tree — which includes `.syscribe.toml` and the
  `.syscribe/results.json` verdict sidecar — and every `[repos]` peer model root loaded from the
  config. `.git/` (and the other VCS directories `.hg/`, `.svn/`), `.syscribe/cache/`, and
  editor temp/swap files (`*.swp`/`*.swo`/`*.swx`, `*~`, `.#*`, Vim's `4913` probe, `*.tmp`)
  are ignored.
- **Debounce + fingerprint.** A burst of file events is debounced (~250 ms quiet period). The
  server then fingerprints the model inputs — relative path, size and modification time of
  every `.md`, `.sysml`, `.kerml` and `.rhai` file, `.syscribe.toml`, `.sysmlignore`,
  `.syscribe/results.json`, and every file under a `foreignFormat:`/`annotationFormat:`
  package directory — and reloads only if that differs from the fingerprint recorded when the
  current store was loaded. The server's own committed writes (which already rebuild the store,
  REQ-TRS-MCP-002) and events on irrelevant files therefore never cause a redundant reload.
- **Off-lock reload.** The fresh store is built without holding the store lock and swapped in
  under the write lock, so read tools are never blocked by a walk. If another reload landed in
  the meantime the watcher re-evaluates instead of overwriting it.
- **Half-saved files.** A walk that fails, or that would introduce a frontmatter parse failure
  on a file that parsed in the current store (unparseable YAML, or an empty / unterminated
  frontmatter), is deferred: the current store is kept, a warning logging message
  `{"event":"reload_deferred","source":"watch",…}` is sent (and printed to stderr), and the
  reload is retried on the next file event. The explicit `reload` tool always loads.
- **Notification.** After a successful automatic reload the server sends the logging message
  `{"event":"reload","source":"watch","count":N}` and `notifications/resources/list_changed`,
  as the `reload` tool does (REQ-TRS-MCP-025).
- **Control.** `syscribe mcp --no-watch` disables watching; `--read-only` servers still watch.
  The `reload` tool stays. The `initialize` instructions say the server reloads automatically.
- **Best-effort and clean shutdown.** If the watcher cannot be created (e.g. the inotify watch
  limit), the server logs once and keeps serving without it. The watcher never keeps the
  process alive: the server exits when stdin closes.

## Rationale

REQ-TRS-MCP-002/025 originally left external edits to the explicit `reload` tool. In practice
agents edit model files directly (or a human edits alongside them) and forget to call
`reload`, then act on a stale graph (GH #181). The web server already watches its model with
the `notify` crate; the MCP server now does the same.
