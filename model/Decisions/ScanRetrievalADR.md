---
type: ADR
id: ADR-SYS-SCAN-001
name: "Tier B LLM-scan retrieval: a compact digest distinct from export, and in-memory BM25 full-text (no tantivy, no persisted index)"
status: accepted
tags:
  - stats
  - search
  - reporting
  - mcp
  - tooling
---

## Context

Tier A (`stats`, `ADR-SYS-STATS-001`) gives an LLM the *shape* of a large requirement
corpus. Tier B adds the next two hops for a model with tens of thousands of
requirements:

1. **Bulk read** — after narrowing with `stats` facets, the LLM wants to *dump the
   slice*: one compact line per requirement, token-budgeted, paginated
   (`REQ-TRS-OUT-022`).
2. **Topical retrieval** — the LLM wants to *find* requirements about a topic, ranked by
   relevance (`REQ-TRS-SEARCH-001`).

Two capabilities already sit nearby: `export --ndjson` (a full per-element dump) and the
MCP `search` tool (fuzzy identifier match with an unranked body substring fallback).
Neither covers Tier B.

## Decision

### 1. `digest` is a new compact view, not a mode of `export`

`export` stays the **full** dump (typed frontmatter + computed reverse indices) for
tools that reconstruct the graph. `digest` is a **separate** command emitting only
`{id, name, status, reqDomain, sil?, asil?, text, verified}` per native `Requirement`
— ~30 tokens/row, NDJSON by default, `{total, offset, rows}` under `--json`, cursor-paged
with `--limit`/`--offset`. It reuses the `stats` scoping (the `--where` predicate, the
tag/status filters, the `--config` projection lens) and the coverage notion of
`verified`. Keeping it separate avoids overloading `export` with a token-budget concern
and lets the two evolve independently.

### 2. Full-text search is in-memory BM25 — no tantivy, no persisted index

`search-text` / the MCP `search_text` tool build an **in-memory inverted index** over the
tokenised element bodies (+ `name`) **at load** and score with Okapi BM25 (`k1`/`b`),
returning ranked results with a snippet marking the first hit. Deliberate deviations from
the original "drop in tantivy, cache to disk keyed by content hash" sketch:

- **No external search engine.** tantivy pulls a large transitive dependency tree and a
  segment/file format; the project's ethos is a single, offline, pure-Rust binary (it
  *vendors* batsat rather than depend on a SAT crate — `ADR-FM-002`). An inverted index
  over the model's short documents is ~150 lines of dependency-free Rust.
- **No persisted index file.** Building the index over the whole model at load is
  sub-second at the target scale, and the server already rebuilds the store on file
  change (the `notify` watcher). A disk cache keyed by content hash is a latency
  optimisation we can add later if load time ever bites; persisting an index would add
  cache-invalidation surface and an on-disk artifact to `.gitignore` for no present gain.
- **The existing `search` tool is unchanged.** `search_text` is the relevance-ranked
  *complement* (BM25 over bodies), not a replacement for identifier fuzzy-match.

### 3. Both are first-class on CLI and MCP

Each capability is a CLI subcommand *and* a first-class read-only MCP tool returning the
same JSON as `--json`; `digest` is additionally on the `run_report` allow-list. This
matches the Tier A pattern (`stats`) and the project convention that the CLI `--json`
contract is the source of truth an MCP tool mirrors.

## Rationale

Reusing the Tier A scoping keeps one narrowing vocabulary across `stats` → `digest`, so
an LLM applies the same filters at each hop. Choosing an in-memory BM25 index over
tantivy trades a marginal cold-start optimisation for the single-binary/offline property
the rest of the tool guarantees — the right trade for a validator/CLI that must run
anywhere with no services. BM25 (not the current unranked `contains`) is what makes
topical retrieval usable: rare terms and dense matches rank first, long documents are
length-normalised.

## Consequences

- New `digest.rs` (compact row producer, shared by CLI + MCP) and `ftsearch.rs`
  (tokeniser + inverted index + BM25 + snippet, shared by CLI + MCP). Both borrow the
  `stats`/`projection` scoping; neither re-implements coverage or parsing.
- `digest` numbers (`verified`) track the coverage definition automatically.
- Full-text results are ranked and snippeted; a change to tokenisation is localised to
  `ftsearch.rs`.
- Deferred to a later tier: a persisted/content-hash-cached index, phrase/AND queries,
  and embeddings/semantic clustering (Tier C).
