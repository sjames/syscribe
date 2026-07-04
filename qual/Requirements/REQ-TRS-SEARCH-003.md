---
id: REQ-TRS-SEARCH-003
type: Requirement
name: Tool shall group requirements into topical clusters via TF-IDF cosine k-means (clusters command)
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `clusters` subcommand
(`syscribe -m <root> clusters [--json] [--k <N>] [--type <T>] [--config <CONF>]`)
that groups elements into **topical clusters** by the similarity of their normative text,
so an LLM can see the **cross-package** themes of a corpus (e.g. "safety-shutdown"
requirements scattered across several subsystems) that a package-by-package or
keyword-by-keyword view misses. This is the semantic-grouping layer of Tier C.

## Clustering method (deterministic, offline)

Each element (by default native `Requirement`; `--type <T>` selects another type)
**shall** be vectorised as its **TF-IDF vector** over the element corpus (the tokeniser
of [[REQ-TRS-SEARCH-001]]), and clustered by **cosine similarity** using **k-means** with
`k` = `--k` (default `min(8, element_count)`). The tool **shall not** use neural
embeddings, an external model, or the network — bag-of-words TF-IDF cosine keeps the tool
**offline, single-binary and deterministic** (per ADR-SYS-SCAN-002; neural embeddings are
deferred as a possible opt-in). To make the result **reproducible**, centroid
initialisation **shall** be deterministic (a farthest-first traversal seeded from a fixed
element order) — no random seed — so repeated runs on the same model yield the same
clusters.

## Cluster content & output

Each cluster **shall** report its **size**, a **label** (its centroid's top TF-IDF
terms), and its **member** element ids. `--config <CONF|features>` **shall** project onto
a variant before clustering; an unresolvable `--config`, or a `--k` less than 1, is a
usage error (exit `1`); `--k` **shall** be clamped to the element count. The default
output **shall** be, per cluster, a label line and its members; `--json` **shall** emit
`{ "k": <n>, "clusters": [ { "label": [terms], "size", "members": [ids] } … ] }`.
`clusters` **shall** be reachable as a first-class read-only MCP tool `clusters`.

**Source:** user request — Tier C of LLM-scale corpus scanning (cross-package semantic
clustering). Read-only; deterministic; offline (TF-IDF cosine, no embeddings).

**Acceptance criteria:**

- `syscribe -m <root> clusters --k 2` prints 2 clusters, each with a term label and its
  member ids; every element appears in exactly one cluster.
- `clusters --json` emits `{ k, clusters: [ { label, size, members } … ] }`, valid JSON;
  the sizes sum to the number of clustered elements.
- Running `clusters` twice on an unchanged model yields **identical** clusters
  (deterministic initialisation — no random seed).
- Two elements whose text shares a distinctive vocabulary land in the same cluster more
  readily than two elements with disjoint vocabulary (cosine grouping).
- `--k 0` exits `1`; `--k` greater than the element count is clamped; an unresolvable
  `--config` exits `1`.
- The MCP `clusters` tool returns the same document as `clusters --json`, read-only.
