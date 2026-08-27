---
type: ADR
id: ADR-SYS-ANNOTATE-001
name: "Annotated-source ingestion: in-process comment-marker scan, reusing the frontmatter grammar"
status: accepted
tags:
  - annotations
  - traceability
  - interop
---

## Context

`ADR-SYS-PLUGIN-002` solved "a subsystem is authored in some genuinely custom notation" by handing a
package's subtree to an external stdio-subprocess plugin. That is not the problem this ADR addresses.
Here the source is *ordinary code* — C, Python, Verilog, whatever a team already writes and compiles
— and the goal is for that code to self-declare its place in the traceability graph, at the point of
implementation, instead of the model authoring the link by hand.

`§12.8` already closes this loop in one direction: a `Part`/`PartDef` may carry `implementedBy:`
pointing at its source artifact(s). That is a *pull* — the model reaches down into code. What is
missing is the *push*: code asserting "I satisfy `REQ-100`" or "I verify `TC-200`" directly beside
the function/module that does it, discovered by Syscribe rather than hand-transcribed into a `.md`
file elsewhere. Teams that already annotate source for Doxygen, Sphinx-Needs, or similar tools expect
this shape of workflow; nothing in Syscribe today supports it.

No new merge mechanism is needed — FMEA/TARA row explosion, feature-model tree explosion, stdio-
plugin envelopes, and native SysMLv2 ingestion all already merge synthesized content into the graph
as ordinary `RawElement`s, resolvable by every cross-reference kind with zero extra work. Reusing
that is a solved problem. The only genuinely new problem is *finding* a marker comment inside a file
whose comment syntax Syscribe doesn't otherwise know, since that varies per language — a small,
regex-shaped problem with no need for a parser, a subprocess, or any code execution at all.

## Decision

A package `_index.md` may declare `annotationFormat: <alias>` plus its scan parameters **inline**:

```yaml
type: Package
name: FirmwareEngineCtrl
annotationFormat: c-linecomment
marker: '//\s*@syscribe\b'
include: ["**/*.c", "**/*.h"]
exclude: ["**/vendor/**"]
```

This is **full-subtree takeover**, mirroring `foreignFormat:`: only the `_index.md` itself stays a
native element (name, doc body, containment entry); every other file under that directory is excluded
from native Markdown parsing. Every file matching `include` (minus `exclude`) is scanned for markers.

**Marker detection.** `marker` is a regex that finds the *start* line of a marker comment. For a
line-comment style, every contiguous line immediately following that shares the same comment-prefix
pattern is accumulated into the same block, stopping at the first non-matching line. For a
block-comment style the block instead runs to the style's closing delimiter. The marker match and
each line's comment-prefix are stripped, leaving a plain text block.

**Content grammar.** That text block is parsed as literal YAML using the exact same frontmatter
schema every native `.md` file uses — no second grammar to define, spec, or teach. Every existing
field and cross-reference kind (`satisfies:`, `verifies:`, `derivedFrom:`, `allocatedTo:`,
`custom_fields`, everything) works immediately, and the synthesized element merges into the graph
exactly as any other synthesized `RawElement` does.

```c
// @syscribe
// type: Part
// id: PART-ENGINE-CTRL
// name: Engine Controller (firmware)
// satisfies: [REQ-100]
// verifies: [TC-200]
```

**Identity.** `id`/`name` is mandatory and explicit on every marker, following that element's
declared `type:`'s ordinary identity rules — never derived from file path or line number. Source
location (`<file>:<line>`) is captured automatically as provenance/`file_path`, and is also used to
**auto-fill `implementedBy:`** when the marker doesn't set it explicitly, closing `§12.8`'s trace leg
for free — the marker's own location already answers "where is this implemented." An explicit
`implementedBy:` in the marker, if given, always wins over the auto-filled value.

**Qname.** `<owning package qname>::<marker's id or name segment>`, the same rule any other
synthesized `RawElement` nested under its owning package already follows.

## Rationale

Seven sub-decisions, each with a rejected alternative:

1. **Reuse the frontmatter grammar verbatim inside the marker**, rather than a compact custom tag
   syntax (`@satisfies REQ-100`, Doxygen-style). *Rejected:* a terser bespoke syntax reads better
   inline for the single-field case, but doesn't generalize — every additional field
   (`custom_fields`, multi-target lists, rationale text) would need its own ad hoc extension,
   duplicating a grammar this project already has, tests, and documents. One grammar, two carriers
   (file frontmatter, comment block) is strictly less surface area than two grammars.
2. **In-process regex scan, not a subprocess.** *Rejected:* reusing the stdio-plugin transport for
   this too — but that would mean shipping a parser binary just to find `//` comments and hand YAML
   back, all the operational overhead of `ADR-SYS-PLUGIN-002` with none of its benefit (a subprocess
   buys you a genuinely custom notation; here the notation is fixed and is already Syscribe's own).
   Also strictly safer: no code execution at all, so this mechanism needs none of that ADR's explicit
   trust-delegation framing.
3. **`annotationFormat:` config lives inline on `_index.md`**, not indirected through
   `.syscribe.toml` (contrast `[plugins.<alias>]`). *Rejected:* a `.syscribe.toml`-indirected alias
   mirroring plugins exactly — but a plugin alias exists to name a *command*, something with real
   operator-trust implications worth centralizing and reviewing separately from any one package's
   frontmatter. A marker regex and a glob list are inert data with no equivalent trust surface;
   forcing indirection through project config for every package that wants this adds a lookup hop for
   no safety benefit. A future revision can still layer reusable named presets on top (see
   Consequences) without revisiting this choice.
4. **Full-subtree takeover, not a layered/overlay model** (mirrors `foreignFormat:`). *Rejected:*
   letting scanned markers coexist with hand-authored `.md` files in the same package — more
   flexible, but reopens the dual-source-of-truth question `ADR-SYS-PLUGIN-002` deliberately closed.
   `E108` already answers what happens when a native element and a scanned marker collide across
   *different* packages; it is not obviously the right answer for two sources inside the *same*
   package coexisting by design. Consistency with the existing precedent wins; revisit only if a real
   use case demands mixing.
5. **`id`/`name` mandatory and explicit on every marker**, never derived from file path or line
   number. *Rejected:* auto-deriving from `<file-stem>_L<line>` — attractive for zero-config markers,
   but unstable: an unrelated edit earlier in the file shifts every line number below it, silently
   renaming every marker's identity, which then reads as spurious churn in every downstream reference
   and in `suspect` baselines. Explicit ids cost one line and buy real stability — the same tradeoff
   every other id-identified type in this format already makes.
6. **Auto-fill `implementedBy:` from marker provenance when unset.** *Rejected:* always requiring it
   explicit — technically more uniform with hand-authored elements, but redundant by construction
   here: the whole reason the marker exists is that its location already *is* the answer to "where is
   this implemented," so asking the author to retype it serves no purpose. An explicit
   `implementedBy:`, if given, still overrides the auto-filled value.
7. **New code range `E560`+/`W560`+** (confirmed unused on `main`). *Rejected:* reusing
   `E550`/`E551`/`W550`–`553` — those are `ADR-SYS-PLUGIN-002`'s own dedicated range; reusing them
   would misattribute an annotated-source finding to the stdio-plugin mechanism to anyone grepping a
   validation report, the same reasoning that ADR gave for not reusing the WASM-plugin range.

## Consequences

- A model with no package declaring `annotationFormat:` is completely unaffected.
- A malformed marker block (regex matched but the accumulated text isn't valid YAML, or is valid YAML
  but not a legal `RawElement` for its declared `type:`) degrades to "zero elements from that marker
  plus one warning finding," never aborting the rest of `validate` — the same graceful-degradation
  posture `ADR-SYS-PLUGIN-002` established.
- `E108` (duplicate qualified name, any origin) already covers a marker-declared id colliding with
  any other element, native or foreign-sourced — no new collision code needed.
- `§12.8`'s `W023` ("missing local `implementedBy:` path") is structurally unreachable for a marker-
  synthesized element once auto-fill lands, since the element cannot exist without a source location.
- Every generic cross-reference kind (`satisfies:`, `derivedFrom:`, `allocatedTo:`, `typedBy:`,
  `supertype:`) works bidirectionally between a marker-synthesized element and the native model with
  zero extra work, exactly as `ADR-SYS-PLUGIN-002` found for plugin-synthesized elements. `verifies:`
  is the one field with a hard-gated target list (`E104`) and will need the same
  `synthesized_qnames`-style provenance widening that ADR's Addendum gave plugin-synthesized elements
  — tracked as Phase 1 work here, not deferred, since it is a known, already-solved shape of fix.
- **Phasing.** Phase 1 — shipped, see Addendum below: core mechanism (marker detection +
  frontmatter-block parsing + merge), the `E104` widening above, an `annotations scan
  <qname-or-label> --dry-run` CLI verb (mirrors `plugins run <alias> --dry-run`), the codes below,
  one worked example (`examples/annotated-source/c-firmware/`), and this document plus
  `docs/model-guide/annotated-source.md`. **Deferred, not built here:** reusable named marker presets
  for common comment styles (`//`, `#`, `--`, `<!-- -->`, `/* */`) so most projects never hand-write a
  regex; block-comment accumulation beyond the simple "closing delimiter" rule (nested block comments,
  language-specific escaping); an `annotations list` CLI verb (mirrors `repos list`/deferred `plugins
  list`).

## New validation codes

| Code | Meaning |
|---|---|
| `E560` | `annotationFormat:` malformed (missing `marker`, `marker` not a valid regex, `include` empty/malformed) |
| `E561` | A marker's accumulated block is not valid YAML |
| `W560` | A marker's YAML is valid but not a legal `RawElement` for its declared `type:` — element dropped |
| `W561` | A marker declares no `type:`/no resolvable identity — element dropped |
| `W562` | `annotationFormat:`/`marker`/`include`/`exclude` present on a non-`Package` type, or on a `Package` that also declares `foreignFormat:`/`sysmlSubmodel:` (mechanisms are mutually exclusive per package) |
| `W563` | A marker's `implementedBy:` was auto-filled (informational — confirms the auto-fill fired; suppressible) |

## Addendum: implementation notes (Phase 1 shipped)

Built as `crates/syscribe-model/src/annotations.rs`, hooked into `walker::walk_model` immediately
after `plugins::apply_foreign_plugins`, plus `crates/syscribe/src/annotations.rs` for the CLI verb.
Three design points sharpened during implementation, none changing the Decision above:

1. **`doc:` is a documented exception to "literal frontmatter, no second grammar."**
   `RawFrontmatter` has no `doc` field — a native file's doc body is the Markdown *below* its
   frontmatter, not part of it — but a marker has no equivalent separate region. A top-level `doc:`
   key in the marker block is read out and becomes the synthesized element's documentation body,
   removed from the YAML before `RawFrontmatter` deserialization (left in place, it would land in
   `RawFrontmatter`'s `#[serde(flatten)] extra` catch-all and spuriously raise `W047`, "unrecognized
   frontmatter field"). Mirrors `plugins::envelope::EnvelopeElement`'s own separate `doc` field for
   the identical reason — plugins solved this by keeping `doc` outside the frontmatter object
   entirely at the JSON-envelope level; a marker has only one flat YAML object to work with, so the
   same effect is achieved by extraction instead.
2. **A new derive-findings code needs registering in *two* places, not one.** `RawElement.derive_findings`
   entries flow through a validator-side allowlist (`validator.rs`, the match arm that maps a code
   string to a `&'static str` or falls back to a sentinel `"E000"`) before becoming real `Finding`s —
   a code used only in the producing module (as `E560`/`E561`/`W560`–`563` initially were) silently
   downgrades to `E000` in every validation report. Same allowlist stdio plugins' `E550`/`E551`/
   `W550`–`553` already had to join; worth calling out here since it is easy to miss and produces no
   compile error, only a wrong code at runtime — caught by exercising the worked example end to end,
   not by the unit tests, which stayed at the module boundary.
3. **A `RawElement.file_path` is already a real, walkable filesystem path — never model-root-relative.**
   `walker::walk_model` builds it by displaying the `Path` `WalkDir` yields when walking `model_root`
   itself, so it already carries `model_root` as a prefix. `plugins::PluginPackage::dir` (and this
   module's `AnnotationPackage::dir`) inherit that same shape by construction. A first pass here
   mistakenly re-joined `model_root` onto `pkg.dir` before walking it, which happened to make the
   module's own from-scratch unit tests pass (they hand-build `RawElement`s with bare, unprefixed
   relative `file_path` strings, unlike the real walker's output) while silently breaking the real
   pipeline — caught the same way as point 2, by running the worked example rather than trusting unit
   tests alone. Fixed by using `pkg.dir` directly and correcting the unit tests to build `file_path`
   the way the real walker does (root-prefixed), rather than adding a compensating join.
