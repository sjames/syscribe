# plugins — WASM foreign-format ingestion (ADR-SYS-PLUGIN-001)

Let a directory inside the model tree be authored in a different text-based
modeling methodology (plain SysMLv2 textual notation, or a user-defined DSL),
parsed by a sandboxed WASM plugin (authored in TypeScript, compiled via
`extism-js`), and merged into the Syscribe graph as first-class elements — so
native `Requirement`/`Allocation`/`derivedFrom:`/`satisfies:`/`verifies:` can
reference them exactly like any hand-authored element.

Read-only ingestion: the foreign folder stays authoritative and is edited by
its own native tooling, never by Syscribe's web UI or mutate commands.

## Configuration (`.syscribe.toml`)

```toml
[plugins.sysmlv2]
wasm = ".syscribe/plugins/sysmlv2-parser.wasm"
timeout_ms = 5000            # optional, default 5000
memory_max_bytes = 67108864  # optional, default 64 MiB
```

## Marking a package foreign (`_index.md`)

```yaml
type: Package
name: SysML2Legacy
foreignFormat: sysmlv2   # must match a [plugins.<alias>] key
```

Only the `_index.md` itself stays a native element; every other file under
that directory is excluded from native parsing and supplied by the plugin.

## Subcommands

```
plugins run <alias> --dry-run   # invoke one plugin, print its raw envelope JSON — no merge
```

Plugin execution itself is automatic: every command that loads the model
(`validate`, `show`, the web server, MCP, …) runs configured plugins as part
of the normal walk, with zero extra steps.

## Caching

A plugin's output is cached at `.syscribe/cache/plugins.json`, keyed by a
content hash of the compiled `.wasm` module plus every file in the foreign
subtree — an unchanged pair skips re-invocation entirely. Only successful
runs are cached (an execution failure always retries); `plugins run
--dry-run` always bypasses the cache.

## Validation

| Code | Condition |
|---|---|
| `E108` | Duplicate qualified name (origin-agnostic — also catches two native files that happen to collide) |
| `E530` | `[plugins.<alias>].wasm` path does not exist on disk |
| `E532` | `foreignFormat: <alias>` has no matching `[plugins.<alias>]` entry in `.syscribe.toml` |
| `W530` | Plugin execution failed — load error, trap, panic, or timeout — that package contributes zero elements this run |
| `W532` | Plugin returned malformed/schema-invalid JSON, or reported its own parse diagnostics |
| `W533` | One element from the plugin failed to map onto `RawFrontmatter` — that element is dropped, siblings kept |
| `W534` | One element's `type:` isn't a recognised element type — that element is dropped |

A plugin execution failure never aborts the rest of validation: it downgrades
that package's contribution to zero elements plus one `W53x` finding, exactly
like an unresolvable multi-repo `ref:` degrades to `RefState::Unknown` rather
than failing the run. Gate CI on any of these with `validate --deny W530`, etc.

## Sandboxing

Plugins run under `extism`/`wasmtime` with no filesystem or network access of
their own — no WASI preopens, `allowed_hosts` is always empty. A plugin reads
its declared subtree only through three host-provided functions (`fs_read`,
`fs_list_dir`, `fs_exists`), each of which canonicalizes the requested path and
rejects anything that resolves outside that subtree before touching disk.
Execution is bounded by `timeout_ms` (wall-clock) and a generous fixed
instruction-fuel ceiling as defense in depth.

## Authoring a plugin

See `examples/wasm-plugins/sysmlv2-toy/` for a complete, buildable example: a
TypeScript plugin (via `@extism/js-pdk`) that recognises a tiny SysMLv2-textual
subset and emits `PartDef`/`RequirementDef` elements. `npm run build` produces
the `.wasm` (requires `extism-js` and Binaryen's `wasm-opt`/`wasm-merge` on
`PATH`). The envelope a plugin's `parse` export must return:

```json
{
  "elements": [
    { "qname": "PressureSensor", "type": "PartDef", "name": "PressureSensor", "doc": "..." },
    { "qname": "SamplingRate", "type": "RequirementDef", "id": "REQ-TOY-001", "doc": "..." }
  ],
  "diagnostics": []
}
```

`qname` is joined onto the owning package's qualified name (`SysML2Legacy::PressureSensor`
above). Any extra keys on an element flow straight into its frontmatter, same
as hand-authored YAML — including `custom_fields` for anything DSL-specific
that doesn't map onto a built-in field.
