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
| `W101` | An SVG `sysml:ref="…"` attribute does not resolve to a model element. |
| `W102` | A local Markdown image/diagram embed path (`![](path)` or `<img src="path">`) does not exist on disk. |

- Mermaid qualified-name resolution **shall** be scoped to ` ```mermaid ` blocks; qualified
  names in **prose** **shall not** be resolved (no false-positive regression — prose qnames
  are deliberately ignored, as for `W099`).
- An SVG with **no** `sysml:ref` attributes (hand-authored / non-syscribe) **shall** produce
  no `W101` findings.
- Remote URIs (`http(s)://`, `data:`, `#…`) in image embeds **shall** be accepted as
  external (no `W102`).
- `lint-docs` **shall** scan `.svg` files (in addition to `.md`) when given a path or
  directory.
- The new findings **shall** appear in both text and `--json` output (shape
  `file`/`line`/`code`/`ref`) and **shall** be gateable like other warnings.

**Source:** GH #74. Read-only; extends the existing `lint-docs` diagnostics.

**Acceptance criteria:**

- A Mermaid node naming a non-existent qualified name emits `W100`; a resolving one does not.
- A stale SVG `sysml:ref` emits `W101`; an SVG with no `sysml:ref` emits none; a resolving
  ref does not.
- A `![](missing.svg)` embed emits `W102`; an existing path and an `https://…` embed emit
  none.
- A qualified name in prose (outside a Mermaid block) emits nothing.
