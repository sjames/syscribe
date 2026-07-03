---
type: ADR
id: ADR-SYS-SCAN-002
name: "Tier C content digest is deterministic and offline: extractive (not LLM) summaries with a content-hash cache, TF-IDF topics, and TF-IDF-cosine clustering (not neural embeddings)"
status: accepted
tags:
  - stats
  - search
  - reporting
  - mcp
  - tooling
---

## Context

Tiers A/B (`ADR-SYS-STATS-001`, `ADR-SYS-SCAN-001`) give an LLM the *shape* of a large
requirement corpus and let it dump/retrieve slices. Tier C is about conveying **what the
requirements mean**: a hierarchical content digest (`summarize`, `REQ-TRS-OUT-023`),
per-package distinctive keywords (`topics`, `REQ-TRS-SEARCH-002`), and cross-package
topical clusters (`clusters`, `REQ-TRS-SEARCH-003`).

The obvious implementations pull the tool away from its core guarantees. "Summarize each
requirement" invites an LLM call; "semantic clustering" invites neural embeddings. Both
break the property the rest of syscribe holds: a **single, offline, deterministic,
pure-Rust binary** that runs anywhere with no services, no API keys, and reproducible
output (it *vendors* batsat rather than depend on a SAT crate — `ADR-FM-002`; Tier B chose
in-memory BM25 over tantivy for the same reason).

## Decision

Tier C stays **deterministic and offline**. No component calls an external model or the
network.

### 1. `summarize` is extractive, not abstractive

The per-package summary is composed from data the tool already computes — a requirement
count, the `stats` status split, the `topics` distinctive terms as an "about" label, and
the `digest` one-line extract of a bounded set of representative requirements — rolled up
bottom-up through the package hierarchy. The tool **does not write prose**; it emits a
structured extract that an LLM client can read (and, if it wishes, abstractively
re-summarize). This keeps `summarize` deterministic, testable, and free of any model
dependency.

### 2. `summarize` caches per package, keyed by a content hash

Each package's computed summary is cached under `<model_root>/.syscribe/cache/summaries.json`
keyed by a hash of that package's requirement identities and bodies. An unchanged package
is served from cache; a changed one recomputes (incremental — only changed subtrees). The
output is **identical** whether cached or recomputed (the cache is pure performance).
`.syscribe/cache/` is already git-ignored. The hash uses the standard-library hasher
(non-cryptographic — a hash collision or a std-version change at worst forces one
recompute, never a wrong answer).

### 3. `topics` and `clusters` use TF-IDF, not embeddings

`topics` treats each package as a document (its elements' body + name concatenated) and
reports the top terms by TF-IDF — the vocabulary that distinguishes a package from the
rest of the corpus. `clusters` vectorises each element as its TF-IDF vector and groups by
**cosine similarity** with **k-means**. Deliberate deviation from the "neural embeddings +
k-means" sketch: embedding models are tens of MB, need a download (network) or a bundled
ONNX/candle runtime, and would break the offline single-binary property for a first cut.
Bag-of-words TF-IDF cosine is dependency-free, offline, and good enough to surface
cross-package themes by shared distinctive vocabulary. Neural embeddings are **deferred**
as a possible opt-in (`--embeddings`) later.

### 4. Clustering is reproducible

k-means centroid initialisation is a **deterministic farthest-first traversal** seeded
from a fixed element order (sorted by qualified name) — no random seed — so repeated runs
on the same model yield the same clusters (required by the tests and by cache sanity).

### 5. One shared TF-IDF core; all three are first-class on CLI and MCP

A single `textstats` module owns the tokeniser (shared with `search-text`), the corpus
TF-IDF, top-terms, TF-IDF vectors and cosine. `summarize`/`topics`/`clusters` are each a
CLI subcommand and a read-only MCP tool returning the same JSON as `--json`; `summarize`
is additionally on the `run_report` allow-list.

## Rationale

Choosing extraction over LLM summarization and TF-IDF over embeddings trades a measure of
semantic nuance for the guarantees that make syscribe usable as a CI/validation tool:
determinism (so a cache and tests are meaningful), offline operation (so it runs in any
pipeline), and a single binary (no model artifacts). The structured extract `summarize`
emits is exactly the scaffold an LLM would want to abstractively summarize from, so the
division of labour is clean: the tool does the deterministic aggregation; the model does
the prose.

## Consequences

- New `textstats.rs` (shared TF-IDF/cosine), `summarize.rs`, `topics.rs`, `clusters.rs`;
  `search-text` reuses the shared tokeniser.
- `summarize` output tracks the `stats`/`digest`/`topics` definitions automatically.
- The summary cache is an on-disk artifact under the already-ignored `.syscribe/cache/`.
- Deferred (opt-in, later): neural embeddings for `clusters`, abstractive summaries, and a
  persisted TF-IDF index. Tier C as shipped is the deterministic floor, not the ceiling.
