# Annotated-Source Ingestion (comment markers)

`GUIDE · ANNOTATED-SOURCE` (`ADR-SYS-ANNOTATE-001`)

Ordinary source code — C, Python, Verilog, whatever a team already writes and compiles — can
self-declare its place in the traceability graph, at the point of implementation, instead of the
model authoring the link by hand. A comment in the code becomes a real Syscribe element, discovered
by an in-process regex scan — no external process, no code execution, nothing to install.

This is the *push* counterpart of `§12.8`'s `implementedBy:` (the *pull*, where a `Part` reaches down
into its source). Use whichever direction fits: hand-author `implementedBy:` on the model side, or
let the code declare itself via a marker — both land in the same graph, resolvable by every
cross-reference kind.

Contrast with [stdio-subprocess plugins](stdio-plugins.md) (`ADR-SYS-PLUGIN-002`): that mechanism
hands a whole foreign notation to an external process you write. This one has no process and no
custom notation — the marker's content is literal Syscribe frontmatter YAML, embedded in a comment.
Reach for plugins when the foreign format is genuinely custom; reach for this when the "foreign
format" is just ordinary code you want to annotate.

Everything here is **opt-in**: a model with no package declaring `annotationFormat:` behaves exactly
as before, and none of this runs.

This is **read-only ingestion**. The source file stays authoritative and is edited by its own normal
tooling — Syscribe never writes into it.

---

## 1. Marking a package annotated — `annotationFormat:`

```yaml
---
type: Package
name: Firmware
annotationFormat: c-linecomment
marker: '//\s*@syscribe\b'
include: ["**/*.c", "**/*.h"]
exclude: ["**/vendor/**"]
---
```

Only the `_index.md` itself stays a native element (name, doc body, containment tree entry); every
other file under that directory is excluded from native Markdown parsing. This is full-subtree
takeover, same as `foreignFormat:` — a package is either native `.md` content or scanned content,
never both. `annotationFormat:`/`marker`/`include`/`exclude` on anything other than a `Package`, or
alongside `foreignFormat:`/`sysmlSubmodel:` on the same package, is `W562` (that package's annotation
scan is skipped; the other mechanism still runs normally).

Unlike `foreignFormat:`'s plugin alias, `annotationFormat:`'s value is a **label only** — there is no
`.syscribe.toml` table to look it up in. `marker`/`include`/`exclude` live inline on this same
`_index.md`, since (unlike naming an external command) they carry no operator-trust decision worth
centralizing.

## 2. The marker and accumulation rule

`marker` is a regex matched against each line of every `include`d file. The first match on a line
starts a block. Contiguous following lines that share the same comment prefix as the marker line fold
into the same block, stopping at the first line that doesn't match — or at another line that itself
starts a new marker, so two adjacent markers are never folded into one block even when they share a
comment style. The marker match and each line's comment-prefix are stripped before parsing (relative
indentation *beyond* the prefix is preserved — it can be significant YAML, e.g. a multi-line `doc:`).

```c
// @syscribe
// type: Part
// name: EngineController
// satisfies: [REQ-ENG-100]
```

becomes the plain text:

```yaml
type: Part
name: EngineController
satisfies: [REQ-ENG-100]
```

(`Part` is name-identified, so the marker carries a basic-name `name:` and no `id:` — see §4.)

The comment leader is detected automatically from the marker's own match — `marker: '//\s*@syscribe\b'`
(leader inside the pattern) and `marker: '@syscribe'` (leader is whatever precedes the match on the
line, e.g. `// `) both work, so `marker` can name either the whole comment-plus-token or just the
distinctive token.

## 3. What goes inside a marker — the same frontmatter you already know

There is no second grammar here. A marker's accumulated block is parsed with the exact same
frontmatter schema as any native `.md` file's YAML — every field, every cross-reference kind
(`satisfies:`, `verifies:`, `derivedFrom:`, `allocatedTo:`, `custom_fields`, all of it) works
immediately. If it's valid in a `.md` file's frontmatter, it's valid in a marker.

One exception, and only one: `doc:`. `RawFrontmatter` has no `doc` field — a native file's doc body is
the Markdown *below* its frontmatter — but a marker has no separate region to put one in, so a
top-level `doc:` key is read out of the block and becomes the synthesized element's documentation
body instead of being treated as frontmatter:

```c
// @syscribe
// type: Part
// name: EngineController
// doc: >-
//   Holds and reports the commanded target engine RPM.
```

The block is not valid YAML → `E561`. It's valid YAML but not a legal `RawElement` for its declared
`type:` → `W560`, and that one element is dropped (the rest of the scan still runs). No `type:`, or
no resolvable `id`/`name` → `W561`, also dropped.

## 4. Identity, and `implementedBy:` auto-fill

`id`/`name` is **mandatory and explicit** on every marker — the same identity rule its `type:`
already enforces everywhere else (a `Part`-typed marker needs whatever a hand-authored `Part` needs:
a basic-name `name:`, since `Part` is name-identified; a `Requirement`-typed marker needs a `REQ-*`
`id:`, since `Requirement` is id-identified). Identity is never derived from the file path or line
number: those drift on unrelated edits, and an identity that silently renames itself on every nearby
line-shift would be worse than requiring one extra line up front. The qname suffix is `id:` when
present, else `name:`.

The marker's source location (`<file>:<line>`) is captured automatically and used to **auto-fill
`implementedBy:`** when the marker doesn't set it itself — the marker's own location already answers
"where is this implemented," so `§12.8`'s trace leg closes for free. Set `implementedBy:` explicitly
in the marker to override the auto-filled value. When auto-fill fires the validator emits warning
`W563`, so a validation report always shows where the pointer came from; it counts toward
`--max-warnings`/`--warnings-as-errors` like any warning — set `implementedBy:` explicitly in the
marker to silence it.

The synthesized element's qualified name is `<owning package qname>::<id or name>`, same rule as any
other synthesized element nested under its owning package.

## 5. Validation codes

| Code | Meaning |
|---|---|
| `E560` | `annotationFormat:` malformed — missing `marker`, `marker` isn't a valid regex, or `include` is empty/malformed |
| `E561` | A marker's accumulated block isn't valid YAML |
| `W560` | Valid YAML, but not a legal `RawElement` for its declared `type:` — element dropped |
| `W561` | No `type:`/no resolvable identity in a marker — element dropped |
| `W562` | `annotationFormat:`/`marker`/`include`/`exclude` on a non-`Package`, or combined with `foreignFormat:`/`sysmlSubmodel:` on the same package |
| `W563` | A marker's `implementedBy:` was auto-filled (a warning — set `implementedBy:` in the marker to silence it) |

`E108` (duplicate qualified name, any origin) already covers a marker-declared id/name colliding with
any other element, native or foreign-sourced. A malformed marker never aborts the rest of `validate`
— only that one marker's contribution is dropped; its siblings in the same file, and every other file,
are still scanned.

## 6. CLI

```bash
syscribe -m model/ annotations scan Firmware --dry-run          # by package qname
syscribe -m model/ annotations scan c-linecomment --dry-run     # by annotationFormat: label (first match)
```

Prints a JSON report (`{"elements": [...], "findings": [...]}`) of what a scan of that one package
would produce — no merge, no validation. The fastest loop for authoring `marker`/`include`/`exclude`,
including seeing exactly why a marker was dropped before trusting it inside a full `validate` run.

---

See `ADR-SYS-ANNOTATE-001` (`model/Decisions/AnnotatedSourceADR.md`) for the full design rationale,
including the alternatives considered and why, and
`examples/annotated-source/c-firmware/` for a complete worked example.
