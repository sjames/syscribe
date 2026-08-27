# Annotated-Source Example: C Firmware

A small, standalone model demonstrating annotated-source ingestion
(`ADR-SYS-ANNOTATE-001`, `docs/model-guide/annotated-source.md`). It is a separate model root from
this repository's own `model/` — running validation here never affects that model.

The `Firmware/` package is ordinary C — not a foreign notation, not something a plugin needs to
parse. Its `_index.md` declares `annotationFormat: c-linecomment` with a `marker:` regex and an
`include:` glob; every `.c`/`.h` file under it is scanned in-process for `// @syscribe` comment
markers instead of being handed to an external plugin. Deliberately the simplest possible case, to
show the point of this mechanism against `plugins`: no subprocess, no build step, no new tool to
write — just a regex and a comment convention.

## Running it

```bash
cargo build --workspace   # once, if you haven't already

# Dry-run: scan one package in isolation, see its raw elements (no merge, no validation)
./target/debug/syscribe -m examples/annotated-source/c-firmware/model annotations scan Firmware --dry-run

# Full validate: the scan runs automatically inside walk_model; its synthesized
# elements participate in ordinary traceability checks
./target/debug/syscribe -m examples/annotated-source/c-firmware/model validate

# The synthesized element resolves and traces like any hand-authored element
./target/debug/syscribe -m examples/annotated-source/c-firmware/model show Firmware::EngineController
```

Current output: **0 errors, 3 warnings** — `W563` (confirms `implementedBy:` was auto-filled from the
marker's own source location) and `W002`/`W005` on `REQ-ENGCTL-100` (no active `TestCase` directly
verifies *it*, and it has no `derivedFrom:`; both expected for a minimal top-level demo requirement).

## What each piece does

- `model/Firmware/_index.md` — `annotationFormat: c-linecomment` marks the package; the whole
  `Firmware/` subtree becomes scan-owned (any stray hand-authored `.md` file placed there would be
  stripped, same as this package's own content today).
- `model/Firmware/engine_ctrl.c` — the actual C source, scanned for markers, never parsed by
  Syscribe's native Markdown+YAML parser. Its `// @syscribe` block declares a `Part` named
  `EngineController` that `satisfies: [REQ-ENGCTL-100]` — a link *from* the source *to* the native
  model — and sets `doc:`, which becomes the element's documentation body rather than a frontmatter
  field. No `implementedBy:` is set; the scanner auto-fills it with the marker's own source location.
- `model/Firmware/vendor/thirdparty_hal.c` — a vendored file containing something that *looks* like a
  marker, kept out of the scan by `exclude: ["**/vendor/**"]`. Demonstrates `exclude:` winning over
  `include:`.
- `model/Requirements/REQ-ENGCTL-100.md` — an ordinary native `Requirement`. Nothing about it knows
  or cares that its satisfying element came from a comment marker — `Firmware::EngineController`
  resolves exactly like a hand-authored `Part` would.
- `model/Tests/TC-ENGCTL-001.md` — an ordinary native `TestCase` whose `verifies:` points *into* the
  annotated source, at the marker-synthesized `Firmware::EngineController`. This direction has one
  legality rule (`E104`): the target must actually have been synthesized by an annotation scan, not
  just be of a matching kind — see `docs/model-guide/annotated-source.md` §5.

## Trying the negative paths

Each of these can be reproduced by editing `model/Firmware/_index.md` or `engine_ctrl.c` temporarily:

- Remove `marker:` or empty out `include:` → `E560`, `validate` still exits non-zero cleanly rather
  than crashing (and the subtree's native content, if any, is left untouched since the scan never ran).
- Break the marker regex, e.g. `marker: '(unclosed'` → `E560`.
- Break the marker's YAML, e.g. `// name: [unterminated` → `E561` on that one marker; any other
  well-formed marker in the same file (or elsewhere in the subtree) is still processed.
- Delete `type:` from the marker → `W561`, that element dropped.
- Add `foreignFormat: someplugin` alongside `annotationFormat:` on the same `_index.md` → `W562`,
  annotation scanning skipped for that package (the other mechanism would run normally, error or not).

See `docs/model-guide/annotated-source.md` for the full contract, marker-accumulation rules, and
validation code reference.
