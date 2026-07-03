---
id: REQ-TRS-SEARCH-002
type: Requirement
name: Tool shall extract distinctive per-package keywords via TF-IDF (topics command)
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `topics` subcommand
(`syscribe -m <root> topics [--json] [--top <N>] [--type <T>] [--config <CONF>]`)
that surfaces, per package, the **distinctive keywords** of that package's elements via
**TF-IDF**, so an LLM (or a human) can see *what each package is about* without reading
its contents. This is the cheap topical layer of Tier C: it names the vocabulary that
distinguishes one package from the rest of the corpus.

## TF-IDF over packages

Each **top-level-and-nested package** **shall** be treated as one document formed by
concatenating the normative text (Markdown body + `name`) of the elements it contains
(by default native `Requirement`s; `--type <T>` selects another element type). Over that
package-document corpus the tool **shall** compute, for each package, the **top-N terms**
by TF-IDF (default N = 10), so terms common to every package (low IDF) are demoted and
terms concentrated in one package are surfaced. Tokenisation **shall** be case-insensitive
over alphanumeric runs, identical to `search-text` ([[REQ-TRS-SEARCH-001]]). The
computation **shall** be **deterministic and offline** (no external model or network).

## Scoping & output

`--config <CONF|features>` **shall** project onto a variant before computing (reusing
[[REQ-TRS-PROJ-001]]); an unresolvable `--config` is a usage error (exit `1`). The
default output **shall** be a per-package list of `term (score)`; `--json` **shall** emit
`{ "packages": { "<pkg>": [ { "term", "score" } … ] } }`, terms ordered by descending
score. `topics` **shall** be reachable as a first-class read-only MCP tool `topics`.

**Source:** user request — Tier C of LLM-scale corpus scanning (per-package keyword
extraction). Read-only; deterministic; reuses the `search-text` tokeniser.

**Acceptance criteria:**

- `syscribe -m <root> topics` prints, per package, its top distinctive terms with scores.
- `topics --json` emits `{ packages: { <pkg>: [ {term, score} … ] } }`, valid JSON, terms
  ordered by descending score, at most `--top` per package (default 10).
- A term concentrated in one package (high TF, low DF) outranks a term common to most
  packages (low IDF) in that package's list.
- `topics --type TestCase` computes over TestCases instead of Requirements.
- `topics --config <C>` computes only over the variant; an unresolvable `--config` exits
  `1`.
- The MCP `topics` tool returns the same document as `topics --json`, read-only.
