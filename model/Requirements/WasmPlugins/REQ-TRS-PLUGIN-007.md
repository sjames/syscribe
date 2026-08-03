---
type: Requirement
id: REQ-TRS-PLUGIN-007
name: "A plugin invocation is skipped and served from an on-disk cache when its inputs are unchanged"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLUGIN-000]
breakdownAdr: Decisions::WasmPluginsADR
tags:
  - plugins
  - performance
---

A plugin's raw output shall be cached at `<model_root>/.syscribe/cache/plugins.json`, keyed by a
content hash of everything the output depends on (the compiled `.wasm` module's own bytes, plus
every file under the foreign package's subtree). A `walk_model` call whose foreign content and
plugin binary are both unchanged since the last successful invocation shall skip re-invoking the
plugin entirely and serve the cached output. `syscribe plugins run --dry-run` shall always bypass
the cache — neither reading nor writing it.

## Rationale

Each plugin invocation JIT-compiles its `.wasm` module from scratch — several seconds of CPU for a
QuickJS-sized (~2.4MB) module, uncached between calls. `syscribe-server`'s live-reload re-runs the
full `walk_model` (and therefore every configured plugin) on *any* file change anywhere in the
model, not just under that plugin's own subtree; the guarded-write path additionally does a full
recursive copy of the model into a temp directory before re-validating, which defeats mtime-based
caching outright (copies get fresh mtimes even when content is byte-identical) — hence a
content-hash key rather than a timestamp one. This follows the existing on-disk,
content-hash-keyed cache convention already established by `syscribe summarize`
(`.syscribe/cache/summaries.json`) rather than inventing a new pattern.

## Scope

- Only a *successful* plugin execution is cached. An execution failure (wasm trap, timeout, load
  error) is never cached — it may be transient (system load, a fluke timeout) rather than a
  deterministic property of the (wasm, content) pair, so every such call retries.
- A successful execution that happens to return syntactically-invalid JSON *is* cached — that
  output is still a deterministic function of the same (wasm, content) pair, so re-invoking would
  reproduce the identical result; caching it is not "stale", and it avoids paying the JIT-compile
  cost again just to reconfirm the same malformed answer.
- The cache uses `blake3`, not a weaker/faster hash — a collision would mean silently serving a
  *different* plugin's output into the traceability graph, not merely an unnecessary recompute.
- No eviction/pruning in this first cut (matches `summaries.json`'s own posture); the cache can
  only grow across a project's lifetime. Revisit only if this is shown to matter in practice.
