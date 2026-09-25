---
id: REQ-TRS-MCP-048
type: Requirement
name: The MCP server shall reload its model automatically when the model's input files change on disk, unless started with --no-watch
status: draft
reqDomain: software
verificationMethod: test
---

While serving, `syscribe mcp` **shall** watch the model root (including `.syscribe.toml` and
the `.syscribe/results.json` verdict sidecar) and every `[repos]` peer model root for file
changes, and **shall** make an edit made outside the server (an editor, `git checkout`, another
tool) visible to subsequent tool calls without an explicit `reload`.

- Bursts of file events **shall** be debounced; after the quiet period the server **shall**
  compare a fingerprint (relative path, size, modification time) of the model's input files
  with the fingerprint recorded for the loaded store and reload only if they differ, so the
  server's own committed writes (which already rebuild the store) and events on irrelevant
  files (`.git/`, `.syscribe/cache/`, editor swap/backup files) **shall not** cause a reload.
- The fresh store **shall** be built without holding the store lock and swapped in afterwards,
  so reads are never blocked by the walk.
- A reload that fails, or that would introduce a new frontmatter parse failure (a half-saved
  file), **shall** be deferred: the current store is kept, a warning is logged
  (`{"event":"reload_deferred","source":"watch",…}`), and the reload is retried on the next
  file event.
- After a successful automatic reload the server **shall** send a logging message
  `{"event":"reload","source":"watch","count":N}` and a `notifications/resources/list_changed`
  notification.
- `--no-watch` **shall** disable watching; `--read-only` servers **shall** still watch. The
  `reload` tool **shall** remain available. Watching is best-effort: if the watcher cannot be
  created, the server **shall** log once and keep serving. The watcher **shall not** keep the
  process alive after stdin closes.

**Source:** `REQ-TRS-MCP-048` (product model), GH #181.
