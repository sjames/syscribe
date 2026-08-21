# plugins — stdio-subprocess foreign-format ingestion (ADR-SYS-PLUGIN-002)

Hand a package directory's entire subtree to an external plugin process. The
plugin parses whatever custom, non-SysMLv2 notation lives there and reports
back real Syscribe elements over stdio (JSON on stdin, JSON on stdout) — any
language, no embedded-runtime toolchain required. This is read-only ingestion:
the foreign folder stays authoritative and is edited by its own native
tooling, never by Syscribe's write paths.

## Marking a package foreign (`_index.md`)

```yaml
type: Package
name: Legacy
foreignFormat: toydsl
```

## Declaring the plugin (`.syscribe.toml`)

```toml
[plugins.toydsl]
command = "python3"                 # PATH-resolved name, or a path relative to the model root
args = ["plugins/toydsl_plugin.py"] # optional, default []
timeout_ms = 10000                  # optional, default 10000
```

## Subcommands

```
plugins run <alias> --dry-run   # invoke one plugin live, print its raw envelope JSON
```

No merge, no validation — the fastest loop for a plugin author, including
seeing exactly what came back when it doesn't parse.

## Trust model

A plugin is a plain OS subprocess with full access — no filesystem/network
sandbox. Configuring `[plugins.<alias>].command` means trusting that command,
the same trust level already accepted for `[plantuml] jar`/`plantuml` on
`PATH` and the `[remote]` `sh -c` download hook. The only enforced safety net
is a wall-clock `timeout_ms` kill; there is no memory ceiling.

## Validation

| Code | Condition |
|---|---|
| `E108` | Two elements — any origin — share a qualified name |
| `E550` | `[plugins.<alias>].command` not found on `PATH` / doesn't exist / not executable |
| `E551` | `foreignFormat: <alias>` has no matching `[plugins.<alias>]` entry |
| `W550` | Plugin execution failed — spawn error, non-zero exit, or timeout-killed |
| `W551` | Malformed envelope JSON, or the plugin self-reported its own diagnostics |
| `W552` | One element's frontmatter didn't deserialize — dropped, siblings kept |
| `W553` | One element's `type:` unrecognised — dropped |

A plugin failure never aborts the rest of `validate` — it downgrades only
that package's contribution. Gate CI on any of these with `validate --deny
W550`, etc.

See `docs/model-guide/stdio-plugins.md` and `examples/stdio-plugins/toy-python/`
for the full wire protocol and a worked example.
