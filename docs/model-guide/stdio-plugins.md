# Foreign-Format Plugins (stdio)

`GUIDE · STDIO-PLUGINS`

Other teams and tools already express systems models in other text-based notations — a project's
own domain-specific language, a legacy format, anything not SysMLv2. **stdio-subprocess plugins**
let a directory inside the model tree be authored in one of those notations, parsed by an external
process Syscribe spawns and talks to over stdio, and merged into Syscribe's graph as first-class
elements — so a native `Requirement`/`Allocation`/`derivedFrom:`/`satisfies:`/`verifies:` can
reference them exactly as it would a hand-authored Markdown element (`ADR-SYS-PLUGIN-002`).

Everything here is **opt-in**: a model with no `[plugins]` table behaves exactly as before, and
none of this runs.

This is **read-only ingestion**. The foreign folder stays authoritative and is edited by its own
native tooling — Syscribe never writes into it.

---

## 1. Marking a package foreign — `foreignFormat:`

```yaml
---
type: Package
name: Legacy
foreignFormat: toydsl
---
```

Only the `_index.md` itself stays a native element (name, doc body, containment tree entry); every
other file under that directory is excluded from native parsing and supplied instead by the plugin.

## 2. Declaring the plugin — `[plugins.<alias>]`

```toml
[plugins.toydsl]
command = "python3"                 # PATH-resolved name, or a path relative to the model root
args = ["plugins/toydsl_plugin.py"] # optional, default []
timeout_ms = 10000                  # optional, default 10000
```

The table key is the alias `foreignFormat:` refers to. Plugin execution happens inside the normal
model walk — every command that loads the model (`validate`, `show`, the web server, MCP, the LSP)
runs configured plugins automatically, with no extra step.

## 3. The wire protocol

Syscribe spawns `command` (with `args`, working directory the model root), writes one JSON request
object to its stdin, then closes stdin:

```json
{
  "protocolVersion": 1,
  "alias": "toydsl",
  "packageQname": "Legacy",
  "packageDir": "/abs/path/to/model/Legacy",
  "modelRoot": "/abs/path/to/model"
}
```

The plugin reads whatever files it needs under `packageDir` itself — ordinary filesystem access, no
host-function RPC layer to go through — and writes exactly **one** JSON object to stdout, then
exits `0`:

```json
{
  "elements": [
    { "qname": "PressureSensor", "type": "PartDef", "name": "PressureSensor", "doc": "Measures cabin pressure." },
    { "qname": "SamplingRate", "type": "RequirementDef", "id": "REQ-TOY-001", "doc": "..." }
  ],
  "diagnostics": []
}
```

- `qname` is joined onto the owning package's qualified name (`Legacy::PressureSensor` above).
- `type` must name a recognised element type ([the same inventory](../format/index.md) native `.md`
  files use); an unrecognised value drops that one element with `W553` rather than failing the run.
- Any extra keys (`id`, `name`, `custom_fields`, …) flow straight into the element's frontmatter,
  exactly like hand-authored YAML.
- A `type: Requirement` element with a valid `REQ-*` id opts into the *full* native Requirement
  traceability rule set (breakdown-ADR, leaf-assignment, …), the same as a hand-authored one.
- `diagnostics` is the plugin's own opinion about its source (parse errors, ambiguous constructs);
  it's folded into a single `W551` finding rather than becoming per-plugin dynamic error codes.

**Plugin logging must go to stderr.** stdout is reserved for exactly one envelope object — no
NDJSON, no trailing debug prints mixed in.

## 4. Linking to (and from) the native model

A plugin-emitted element is a real `RawElement`, so it carries the same cross-reference fields a
hand-authored one does — `satisfies:`, `derivedFrom:`, `allocatedTo:`, `typedBy:`, `supertype:`,
`verifies:`, etc. — resolved by Syscribe's normal resolver, by qname or stable id, after the merge.
There is no special syntax and no pre-validation step the plugin needs to perform itself:

```json
{ "qname": "FlowController", "type": "Part", "domain": "software", "satisfies": ["REQ-TOY-100"] }
```

This links **out of** the foreign subtree into the native model — the plugin just needs to know the
target's id or qname, the same way a hand-authored `.md` file would.

Linking **into** the foreign subtree from the native model works too, with one caveat: `verifies:`
targets are checked against a fixed legality rule (`E104`) — a native `Requirement`, or a
requirement/architecture-shaped element (`Part`/`PartDef`/`Attribute`/`Port`/`Connection`/
`Interface`/`Item`/`Allocation`/`RequirementDef`, defs or usages) that was *actually synthesized* by
this mechanism, not merely of a matching kind. A native `TestCase` can `verifies:` a plugin-emitted
`PartDef` for exactly this reason; it cannot `verifies:` an ordinary hand-authored `PartDef` sitting
outside any `foreignFormat:` package — `E104` still rejects that, unchanged. Every other
cross-reference kind (`satisfies:`, `derivedFrom:`, `allocatedTo:`, …) has no such gate in either
direction.

## 5. Trust model

A plugin is a plain OS subprocess with **full access** — filesystem, network, everything the
Syscribe process itself can do. There is no sandbox: configuring `[plugins.<alias>].command` means
trusting that command, the same trust level this codebase already accepts for `[plantuml] jar`/
`plantuml` on `PATH` and the `[remote]` `sh -c` download hook. Only configure a plugin command you
trust, the same way you'd trust any other dev-tool dependency on `PATH`.

- **No `memory_max_bytes`.** Enforcing a memory ceiling on a plain subprocess needs `setrlimit`/
  cgroups/platform-specific work, out of scope here.
- **`timeout_ms` (wall-clock) is the only enforced safety net.** A runaway plugin is killed, not
  memory-bounded. A plugin that deliberately forks a surviving orphan process can, in principle,
  outlive its own kill — a known, accepted limitation of the no-sandbox trade-off, not something
  this mechanism defends against.

(A sandboxed WASM-based alternative was designed separately, `ADR-SYS-PLUGIN-001`, trading this
simplicity for a real filesystem/network sandbox at the cost of a compile-to-WASM toolchain
requirement on every plugin author. It was never shipped; this stdio mechanism is what's built.)

## 6. Validation

| Code | Condition |
|---|---|
| `E108` | Two elements — any origin — share a qualified name (origin-agnostic sibling of `E101`) |
| `E550` | `[plugins.<alias>].command` cannot be resolved — not found on `PATH`, or not executable |
| `E551` | `foreignFormat: <alias>` has no matching `[plugins.<alias>]` entry |
| `W550` | Plugin execution failed (spawn error, non-zero exit, timeout) — that package contributes zero elements this run |
| `W551` | Plugin returned malformed JSON, or reported its own parse diagnostics |
| `W552` | One element's extra frontmatter didn't map cleanly — that element dropped, siblings kept |
| `W553` | One element's `type:` isn't recognised — that element dropped |

A plugin failure never aborts the rest of `validate` — it downgrades only that package's
contribution, the same graceful-degradation posture multi-repo composition's `RefState::Unknown`
already established. Gate CI on any of these with `validate --deny W550`, etc.

## 7. Debugging a plugin

```bash
syscribe -m model/ plugins run toydsl --dry-run
```

Invokes one configured plugin and prints its raw envelope JSON without merging it into the graph or
running validation — the fastest loop for a plugin author, including seeing exactly what came back
when it doesn't parse.

## 8. Authoring a plugin

See [`examples/stdio-plugins/toy-python/`](https://github.com/sjames/syscribe/tree/main/examples/stdio-plugins/toy-python)
for a complete, dependency-free example: a stdlib-only Python script recognising a tiny made-up DSL.
No build step, no toolchain — any language capable of reading stdin and writing stdout works.

## 9. What's not built yet

This is Phase 1 of a phased rollout (`ADR-SYS-PLUGIN-002`). Not yet built:

- **Content-hash caching.** Each `walk_model` re-invokes every configured plugin from scratch —
  fine for a fast script, potentially wasteful under `syscribe-server`'s reload-on-any-change. A
  `.syscribe/cache/plugins.json` cache (mirroring `syscribe summarize`'s convention) is deferred
  follow-on scope.
- **Write-protection for plugin-owned elements** in the web UI's/MCP's mutate routes.
- **`plugins list`** CLI verb (mirrors `repos list`).
