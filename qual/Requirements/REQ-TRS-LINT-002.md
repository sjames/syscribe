---
id: REQ-TRS-LINT-002
type: Requirement
name: lint-docs shall resolve diagram references (Mermaid W100, SVG W101, image embeds W102)
status: draft
reqDomain: software
verificationMethod: test
---

The `lint-docs` command (which today reports unresolvable stable-ID tokens in prose as
`W099`, [[REQ-TRS-LINT-001]]) **shall** additionally validate **diagram** references against
the model (GH #74):

| Code | Condition |
|---|---|
| `W100` | A **qualified name** (`A::B::C`) used inside a fenced ` ```mermaid ` block does not resolve to a model element (by id or qualified name). |
| `W101` | An SVG `sysml:ref="…"` attribute does not resolve to a model element — neither itself nor any `::`-ancestor of it (the diagram shape-ref rule, below). |
| `W102` | A local Markdown image/diagram embed path (`![](path)` or `<img src="path">`) does not exist on disk. |

- Mermaid qualified-name resolution **shall** be scoped to ` ```mermaid ` blocks; qualified
  names in **prose** **shall not** be resolved (no false-positive regression — prose qnames
  are deliberately ignored, as for `W099`).
- `W101` **shall** apply the **same** resolution rule `validate` applies to a Diagram's
  shapes `ref` (`W402`): a `sysml:ref` is accepted when it resolves (by id, qualified name
  or display name) **or** when any `::`-ancestor of it resolves, so a ref naming an inline
  feature of a resolvable element — a port, part usage, sub-state or action step at any
  depth (e.g. `UAV::Power::PowerSystem::battery::powerOut`) — is **not** flagged. A ref with
  no resolvable prefix at all **shall** still raise `W101`. Both checks **shall** share one
  implementation so they cannot drift (GH #172). (`W100` mirrors `validate`'s Mermaid
  `%% ref:` check `W408`, which has no ancestor rule, and so stays an exact lookup.)
- An SVG with **no** `sysml:ref` attributes (hand-authored / non-syscribe) **shall** produce
  no `W101` findings.
- Remote URIs (`http(s)://`, `data:`, `#…`) in image embeds **shall** be accepted as
  external (no `W102`).
- `lint-docs` **shall** scan `.svg` files (in addition to `.md`) when given a path or
  directory.
- The new findings **shall** appear in both text and `--json` output (shape
  `file`/`line`/`code`/`ref`) and **shall** be gateable like other warnings.

**Source:** GH #74; ancestor rule GH #172. Read-only; extends the existing `lint-docs` diagnostics.

**Acceptance criteria:**

- A Mermaid node naming a non-existent qualified name emits `W100`; a resolving one does not.
- A stale SVG `sysml:ref` emits `W101`; an SVG with no `sysml:ref` emits none; a resolving
  ref does not.
- An SVG `sysml:ref` naming a feature of a resolvable element (`Engine::crankshaft`,
  `Engine::crankshaft::flange`) emits no `W101`; one whose every prefix is unknown
  (`Ghost::Thing::port`) still emits `W101` (GH #172).
- A `![](missing.svg)` embed emits `W102`; an existing path and an `https://…` embed emit
  none.
- A qualified name in prose (outside a Mermaid block) emits nothing.
