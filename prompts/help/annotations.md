# annotations — in-process comment-marker ingestion (ADR-SYS-ANNOTATE-001)

Hand a package directory's entire subtree to Syscribe's own in-process
comment-marker scanner instead of the native Markdown+YAML parser. Unlike
`plugins` (`ADR-SYS-PLUGIN-002`), there is no external process and no
`.syscribe.toml` indirection: the source stays ordinary code (C, Python,
Verilog, whatever), and a comment marker embedded in it contains literal
Syscribe frontmatter YAML — the same grammar a hand-authored `.md` file's
frontmatter uses. This is read-only ingestion: the source file stays
authoritative and is edited by its own normal tooling, never by Syscribe's
write paths.

## SYNOPSIS
    syscribe -m <root> annotations scan <qname-or-label> --dry-run

## Marking a package annotated (`_index.md`)

```yaml
type: Package
name: Firmware
annotationFormat: c-linecomment
marker: '//\s*@syscribe\b'
include: ["**/*.c", "**/*.h"]
exclude: ["**/vendor/**"]
```

## The marker itself

```c
// @syscribe
// type: Part
// name: EngineController
// satisfies: [REQ-100]
```

`marker` locates the start line; contiguous comment lines sharing the same
comment-prefix fold into one block, parsed as literal frontmatter YAML. An
element's `id:` (if present) or `name:` becomes its qname segment — the same
identity rule its `type:` already enforces everywhere else. `implementedBy:`
is auto-filled with the marker's own source location when left unset
(`W563`) — the marker's location already answers "where is this
implemented."

## Subcommands

```
annotations scan <qname-or-label> --dry-run   # scan one package in isolation, print its elements
```

`<qname-or-label>` matches a package's qualified name first, then (first
match) its `annotationFormat:` label. No merge, no validation — the fastest
loop for authoring `marker`/`include`/`exclude`.

## Validation

| Code | Condition |
|---|---|
| `E108` | Two elements — any origin — share a qualified name |
| `E560` | `annotationFormat:` malformed — missing/invalid `marker`, or empty `include` |
| `E561` | A marker's accumulated block isn't valid YAML |
| `W560` | Valid YAML, but not a legal element for its declared `type:` — dropped |
| `W561` | No `type:`/no resolvable identity in a marker — dropped |
| `W562` | `annotationFormat:` combined with `foreignFormat:`/`sysmlSubmodel:` on the same package |
| `W563` | A marker's `implementedBy:` was auto-filled (informational) |

A malformed marker never aborts the rest of `validate` — it downgrades only
that one marker's contribution. Gate CI on any of these with `validate --deny
W560`, etc.

See `docs/model-guide/annotated-source.md` and
`examples/annotated-source/c-firmware/` for the full contract and a worked
example.
