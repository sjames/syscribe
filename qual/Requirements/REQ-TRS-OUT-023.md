---
id: REQ-TRS-OUT-023
type: Requirement
name: Tool shall provide a hierarchical summarize command with content-addressed caching for LLM corpus digestion
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `summarize` subcommand
(`syscribe -m <root> summarize [--json] [--scope <qname>] [--depth <N>] [--no-cache] [--config <CONF>]`)
that produces a **bottom-up, per-package hierarchical digest** of the model, so that an
LLM can read a handful of package summaries instead of tens of thousands of requirement
files. This is Tier C of LLM-scale scanning: where `stats` gives the shape and `digest`
dumps the rows, `summarize` conveys **what each part of the model is about**.

## Deterministic (extractive) summarization

Summarization **shall** be **deterministic and offline** — an **extractive** rollup, not
an LLM/abstractive one. The tool **shall not** call any external model or network. The
per-package summary is composed from data the tool already computes plus extractive
sentence selection, so the same model always yields the same summary (a requirement of
the content-hash cache below and of the tests). An LLM client consumes the rolled-up
extract; the tool does not itself write prose.

## Rollup content

For each package (directory node), bottom-up, `summarize` **shall** compute:

- a **requirement count** and a **status split** (reusing the `stats` status facet);
- the package's **top distinctive terms** (the TF-IDF keywords of [[REQ-TRS-SEARCH-002]]),
  as a one-line "about" label;
- a bounded number of **representative requirements** — the one-line extract of each
  (the first body sentence, reusing the `digest` one-line rule);
- the **child package summaries** nested beneath it (the hierarchy).

`--scope <qname>` **shall** restrict the digest to a subtree; `--depth <N>` **shall**
bound the nesting depth reported; `--config <CONF|features>` **shall** project onto a
variant first (reusing [[REQ-TRS-PROJ-001]]). An unresolvable `--scope`/`--config` is a
usage error (exit `1`).

## Content-addressed cache

`summarize` **shall** cache each package's computed summary under
`<model_root>/.syscribe/cache/summaries.json`, keyed by a **content hash** of that
package's inputs (its requirements' identities and bodies). On a subsequent run an
unchanged package **shall** be served from cache (incremental — only changed subtrees
recompute); a changed package **shall** recompute and update its entry. `--no-cache`
**shall** bypass and rewrite. The cache directory is git-ignored (`.syscribe/cache/`).
The **output shall be identical** whether served from cache or recomputed (the cache is
a pure performance optimisation, never a semantic difference).

## Output & exposure

The default output **shall** be an indented text tree; `--json` **shall** emit the
nested structure `{ qname, count, statusSplit, terms, representative, children }`.
`summarize` **shall** be reachable as a first-class read-only MCP tool `summarize` and
via the `run_report` allow-list.

**Source:** user request — Tier C of LLM-scale corpus scanning (hierarchical content
digest). Read-only; deterministic; no external model; aggregates existing data.

**Acceptance criteria:**

- `syscribe -m <root> summarize` prints an indented per-package tree with, for each
  package, a count, a status split, an "about" term line, and representative one-liners.
- `summarize --json` emits the nested `{ qname, count, statusSplit, terms, representative,
  children }` document; it is valid JSON.
- Running `summarize` twice on an unchanged model yields **identical** output, and the
  second run reads `.syscribe/cache/summaries.json` (cache hit); `--no-cache` still
  yields the same output.
- Editing one requirement's body changes only that package's summary (and its ancestors'
  rolled-up view); unrelated packages are unchanged and served from cache.
- `summarize --scope <pkg>` restricts to that subtree; `--depth 1` bounds the nesting; an
  unresolvable `--scope` or `--config` exits `1`.
- The MCP `summarize` tool returns the same document as `summarize --json`, read-only.
