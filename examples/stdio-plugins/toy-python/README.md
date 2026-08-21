# stdio Plugin Example: Toy DSL (Python)

A small, standalone model demonstrating the stdio-subprocess foreign-format
plugin mechanism (`ADR-SYS-PLUGIN-002`, `docs/model-guide/stdio-plugins.md`).
It is a separate model root from this repository's own `model/` — running
validation here never affects that model.

The `Legacy/` package is authored entirely in a made-up "toy DSL" (`.toy`
files, `Legacy/widgets.toy`) instead of Markdown+YAML. Its `_index.md`
declares `foreignFormat: toydsl`; `.syscribe.toml` names `plugin.py` — a
minimal, dependency-free Python script — as the plugin that parses it.
Deliberately zero-toolchain (no build step, no `pip install`) to show the
point of a stdio plugin: any language, no embedded runtime.

## Running it

```bash
cargo build --workspace   # once, if you haven't already

# Dry-run: invoke the plugin directly, see its raw envelope JSON (no merge, no validation)
./target/debug/syscribe -m examples/stdio-plugins/toy-python/model plugins run toydsl --dry-run

# Full validate: the plugin runs automatically inside walk_model; its
# synthesized elements participate in ordinary traceability checks
./target/debug/syscribe -m examples/stdio-plugins/toy-python/model validate

# The synthesized elements resolve and trace like any hand-authored element
./target/debug/syscribe -m examples/stdio-plugins/toy-python/model show Legacy::PressureSensor
./target/debug/syscribe -m examples/stdio-plugins/toy-python/model show Legacy::SamplingRate
```

Current output: **0 errors, 5 warnings** — `W002`/`W005` on `REQ-TOY-100`
(no active TestCase directly verifies *it*, and it has no `derivedFrom:`;
both expected for a minimal top-level demo requirement) and three `W007`s
("defined but never used as a supertype or type", expected since this demo
doesn't wire the synthesized `PartDef`s into any `Part`).

## What each piece does

- `model/Legacy/_index.md` — `foreignFormat: toydsl` marks the package; the
  whole `Legacy/` subtree becomes plugin-owned (any stray hand-authored `.md`
  file placed there would be stripped, same as this package's own content
  today).
- `model/Legacy/widgets.toy` — the actual foreign-format source, parsed by
  the plugin, never by Syscribe's native Markdown+YAML parser. Its
  `FlowController` line carries `satisfies=REQ-TOY-100` — a link *from* the
  foreign model *to* the native one.
- `model/Requirements/REQ-TOY-100.md` — an ordinary native `Requirement`.
  Nothing about it knows or cares that its satisfying element came from a
  plugin — `Legacy::FlowController` resolves exactly like a hand-authored
  `PartDef` would.
- `model/Tests/TC-TOY-001.md` — an ordinary native `TestCase` whose
  `verifies:` points *into* the foreign model, at the plugin-synthesized
  `Legacy::PressureSensor`. This direction has one legality rule (`E104`):
  the target must actually have been synthesized by the plugin, not just be
  of a matching kind — see `docs/model-guide/stdio-plugins.md` §4.
- `model/.syscribe.toml` — `[plugins.toydsl]` names the executable
  (`python3`) and its argument (`../plugin.py`, resolved relative to the
  model root, which is the plugin process's working directory).
- `plugin.py` — reads one JSON request from stdin (`packageDir`, among other
  fields), parses every `*.toy` file it finds there, and writes one JSON
  envelope (`{"elements": [...], "diagnostics": [...]}`) to stdout. All of
  its own logging goes to stderr — stdout is reserved for exactly one
  envelope object.

## Trying the negative paths

Each of these can be reproduced by editing `model/.syscribe.toml` or
`plugin.py` temporarily:

- Rename `[plugins.toydsl]` to `[plugins.wrongalias]` → `E551`, `validate`
  still exits non-zero cleanly rather than crashing.
- Make `plugin.py` `sys.exit(1)` → `W550` (message includes the stderr tail).
- Make `plugin.py` sleep past `timeout_ms` → `W550` (the process is killed,
  not waited out).
- Make `plugin.py` print something that isn't JSON → `W551`.

See `docs/model-guide/stdio-plugins.md` for the full wire protocol, trust
model, and validation code reference.
