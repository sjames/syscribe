# WASM Foreign-Format Plugins

`GUIDE · WASM-PLUGINS`

Other teams and tools already express systems models in other text-based notations — plain SysMLv2 textual notation, or a project's own domain-specific language. **WASM foreign-format plugins** let a directory inside the model tree be authored in one of those notations, parsed by a sandboxed plugin, and merged into Syscribe's graph as first-class elements — so a native `Requirement`/`Allocation`/`derivedFrom:`/`satisfies:`/`verifies:` can reference them exactly as it would a hand-authored Markdown element (`ADR-SYS-PLUGIN-001`).

Everything here is **opt-in**: a model with no `[plugins]` table behaves exactly as before, and none of this runs.

This is **read-only ingestion**. The foreign folder stays authoritative and is edited by its own native tooling — Syscribe never writes into it.

---

## 1. Marking a package foreign — `foreignFormat:`

```yaml
---
type: Package
name: SysML2Legacy
foreignFormat: sysmlv2
---
```

Only the `_index.md` itself stays a native element (name, doc body, containment tree entry); every other file under that directory is excluded from native parsing and supplied instead by the plugin.

## 2. Declaring the plugin — `[plugins.<alias>]`

```toml
[plugins.sysmlv2]
wasm = ".syscribe/plugins/sysmlv2-parser.wasm"
timeout_ms = 5000            # optional, default 5000
memory_max_bytes = 67108864  # optional, default 64 MiB
```

The table key is the alias `foreignFormat:` refers to. At load time, plugin execution happens inside the normal model walk — every command that loads the model (`validate`, `show`, the web server, MCP, the LSP) runs configured plugins automatically, with no extra step.

## 3. The envelope

A plugin's `parse` export returns a JSON object:

```json
{
  "elements": [
    { "qname": "PressureSensor", "type": "PartDef", "name": "PressureSensor", "doc": "Measures cabin pressure." },
    { "qname": "SamplingRate", "type": "RequirementDef", "id": "REQ-TOY-001", "doc": "..." }
  ],
  "diagnostics": []
}
```

- `qname` is joined onto the owning package's qualified name (`SysML2Legacy::PressureSensor` above).
- `type` must name a recognised element type ([the same inventory](../format/index.md) native `.md` files use); an unrecognised value drops that one element with `W534` rather than failing the run.
- Any extra keys flow straight into the element's frontmatter, exactly like hand-authored YAML — including `custom_fields` for anything DSL-specific with no built-in equivalent.
- A `type: Requirement` element with a valid `REQ-*` id opts into the *full* native Requirement traceability rule set (breakdown-ADR, leaf-assignment, …), the same as a hand-authored one. There's no restriction against a plugin emitting native-typed elements — if it's genuinely parsing SysMLv2 `requirement def`, it should get real requirement treatment.
- `diagnostics` is the plugin's own opinion about its source (parse errors, ambiguous constructs); it's folded into a single `W532` finding rather than becoming per-plugin dynamic error codes.

## 4. Sandboxing

A plugin runs under [`extism`](https://extism.org)/`wasmtime` with:

- **No network** — `allowed_hosts` is always empty.
- **No filesystem access of its own** — no WASI preopens. The plugin reads its declared subtree only through three host-provided functions (`fs_read`, `fs_list_dir`, `fs_exists`), each of which canonicalizes the requested path and rejects anything that resolves outside the subtree before touching disk.
- **Bounded execution** — `timeout_ms` (wall-clock) plus a generous fixed instruction-fuel ceiling as defense in depth.

(`extism-js`'s QuickJS-ng runtime exposes no `fs`/`net`/syscall surface to JS/TS at all, so literal WASI preopens aren't achievable for a TypeScript plugin on this toolchain regardless — the host functions give the identical sandboxing property via RPC instead of syscalls.)

## 5. Validation

| Code | Condition |
|---|---|
| `E108` | Two elements — any origin — share a qualified name (a pre-existing gap this work surfaced and closed) |
| `E530` | `[plugins.<alias>].wasm` path does not exist on disk |
| `E532` | `foreignFormat: <alias>` has no matching `[plugins.<alias>]` entry |
| `W530` | Plugin execution failed (load error, trap, panic, timeout) — that package contributes zero elements this run |
| `W532` | Plugin returned malformed JSON, or reported its own parse diagnostics |
| `W533` | One element's extra frontmatter didn't map cleanly — that element dropped, siblings kept |
| `W534` | One element's `type:` isn't recognised — that element dropped |

A plugin failure never aborts the rest of `validate` — it downgrades only that package's contribution, the same graceful-degradation posture multi-repo composition's `RefState::Unknown` already established. Gate CI on any of these with `validate --deny W530`, etc.

## 6. Debugging a plugin

```bash
syscribe -m model/ plugins run sysmlv2 --dry-run
```

Invokes one configured plugin and prints its raw envelope JSON without merging it into the graph or running validation — the fastest loop for a plugin author, including seeing exactly what came back when it doesn't parse.

## 7. Caching

Each plugin invocation JIT-compiles its `.wasm` module from scratch — several seconds of CPU for a QuickJS-sized module, uncached between calls. To avoid paying that on every `walk_model` (which `syscribe-server`'s live-reload triggers on *any* file change anywhere in the model, and which the guarded-write path triggers again via its candidate-copy walk), a plugin's output is cached at `<model_root>/.syscribe/cache/plugins.json`, keyed by a `blake3` hash of the compiled `.wasm` module's own bytes plus every file under the foreign package's subtree — mirroring the same on-disk, content-hash-keyed convention `syscribe summarize` already uses (`.syscribe/cache/summaries.json`).

- **Self-invalidating.** Change any file in the foreign subtree, or rebuild the plugin binary, and the hash changes — no explicit invalidation logic, no staleness window.
- **Only successes are cached.** An execution failure (trap, timeout, load error) is never cached and always retries — it may be a transient fluke of system load, not a deterministic property of the input. A successful run that returns syntactically-invalid JSON *is* cached, since that's still a deterministic function of the same inputs.
- **`plugins run --dry-run` always bypasses the cache**, both reading and writing — a plugin author debugging wants a guaranteed-live run every time.
- No eviction in this first cut — the cache file can only grow across a project's lifetime, same posture as `summaries.json`.

## 8. Authoring a plugin

See [`examples/wasm-plugins/sysmlv2-toy/`](https://github.com/sjames/syscribe/tree/main/examples/wasm-plugins/sysmlv2-toy) for a complete, buildable example: a TypeScript plugin (via `@extism/js-pdk`) recognising a tiny SysMLv2-textual subset.

```bash
cd examples/wasm-plugins/sysmlv2-toy
npm install
npm run build   # esbuild bundle -> extism-js compile -> dist/plugin.wasm
```

Requires `extism-js` and Binaryen's `wasm-opt`/`wasm-merge` on `PATH` (see the [js-pdk install docs](https://github.com/extism/js-pdk)).

## 9. What's not built yet

This is Phase 1 (plus most of Phase 2) of a phased rollout (`ADR-SYS-PLUGIN-001`). Not yet built:

- **Write-protection.** A plugin-owned element is not currently rejected by the web UI's mutate routes beyond incidental path checks — a formal guard is follow-on scope.
- **True fuzzing.** The path-traversal boundary has direct unit tests for the specific cases that matter (`..`, absolute paths, a real symlink escape), but not randomized/property-based adversarial input generation.
- **A second, non-SysML-shaped example plugin**, to pressure-test whether every foreign DSL really maps cleanly onto the existing element-type inventory.
