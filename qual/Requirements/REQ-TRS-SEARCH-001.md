---
id: REQ-TRS-SEARCH-001
type: Requirement
name: Tool shall provide ranked full-text search over normative text with BM25 relevance and snippets
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `search-text` subcommand
(`syscribe -m <root> search-text <query> [--json] [--limit <N>] [--type <T>] [--status <s>] [--config <CONF>]`)
that performs **ranked full-text search** over the normative text of model elements
(the Markdown body plus the `name` label), returning the best-matching elements ordered
by relevance with a highlighted snippet. This closes a gap for LLM-scale scanning: the
existing `search` MCP tool fuzzy-matches identifiers and falls back to an **unranked**
case-insensitive body substring scan (every body hit ties), so "find requirements about
thermal shutdown" cannot be ordered by relevance.

## Ranking

Relevance **shall** be scored with **Okapi BM25** (the standard `k1`/`b` parametrisation)
over an inverted index of the tokenised element bodies, so that rarer query terms and
denser matches rank higher and long documents are length-normalised. A multi-term query
**shall** match documents containing **any** term (OR semantics), ranked by summed term
relevance. Tokenisation **shall** be case-insensitive over alphanumeric runs.

The index **shall** be built **in memory at load** from the parsed model (no external
search engine and no persisted index file in this scope — the single-binary, offline
property is preserved, per the ADR). Building it over the whole model **shall not**
require a network or a separate service.

## Results

Each result **shall** carry the element `id` (when present) and `qualified_name`, its
`type`, the BM25 `score`, and a **snippet**: a bounded window of the body around the
first query-term hit, with the matched term(s) marked (e.g. `**term**`). Results
**shall** be ordered by descending score and bounded by `--limit` (default 10).

## Scoping

`--type <T>` **shall** restrict the search to one element type (e.g. `Requirement`);
`--status <s>` **shall** restrict by `status:`; `--config <CONF|features>` **shall**
project onto a variant before searching (reusing [[REQ-TRS-PROJ-001]]). An unresolvable
`--config` is a usage error (exit `1`). An empty query is a usage error (exit `1`).

## Output

The default output **shall** be a human-readable ranked list (`id`/qname, score,
snippet). With **`--json`**, the output **shall** be one document `{ "total": <n>,
"results": [ { id, qname, type, score, snippet } … ] }` where `total` is the number of
matching documents before `--limit`.

## LLM exposure

Full-text search **shall** be reachable by an MCP client as a **first-class read-only
tool `search_text`** (mirroring the CLI, accepting `query`/`limit`/`type`/`status`/
`config` and returning the ranked `{ total, results }` document). The existing `search`
tool is unchanged; `search_text` is the relevance-ranked complement.

**Source:** user request — Tier B of LLM-scale corpus scanning (ranked full-text
retrieval so an LLM can find topically-relevant requirements without reading every file).
Read-only; no new element types or validation rules.

**Acceptance criteria:**

- `syscribe -m <root> search-text "<term>"` prints a ranked list; the most relevant
  element appears first.
- A document containing a rare query term ranks above one where the term is common /
  diluted by length (BM25 behaviour), and results are ordered by descending `score`.
- Each result carries `id`/`qname`, `type`, `score` and a one-window `snippet` that
  marks the matched term.
- `search-text "<term>" --type Requirement` returns only `Requirement` elements;
  `--status approved` restricts by status; `--config <C>` searches only the variant.
- An empty query exits `1`; an unresolvable `--config` exits `1`.
- `search-text --json` emits one document with `total` and a `results` array ordered by
  score.
- The MCP `search_text` tool returns the same ranked document as `search-text --json`
  for the same arguments and is advertised as read-only.
