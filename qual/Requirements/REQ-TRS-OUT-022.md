---
id: REQ-TRS-OUT-022
type: Requirement
name: Tool shall provide a token-budgeted digest command emitting one compact line per requirement for LLM bulk scanning
status: verified
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `digest` subcommand
(`syscribe -m <root> digest [--json] [--limit <N>] [--offset <N>] [--where <predicate>]... [--status <s>] [--tag <t>] [--config <CONF>]`)
that emits **one compact line per native `Requirement`** — a token-budgeted bulk view
tuned for an LLM to scan tens of thousands of requirements after narrowing with the
`stats` facets ([[REQ-TRS-OUT-021]]). It is the "dump the slice" companion to the
"grasp the shape" digest: `stats` says *how many and where*, `digest` streams the rows.

This is distinct from `export` ([[REQ-TRS-OUT]] issue #2), which dumps the **full**
element (typed frontmatter plus computed reverse indices) and is not token-budgeted.

## Row shape

Each row **shall** be a compact JSON object carrying only:

- `id` (stable `REQ-*`), `name` (label), `status`, `reqDomain`;
- the integrity level when present — `sil` (integer) and/or `asil` (string);
- `text` — a **one-line** summary: the first non-empty line of the Markdown body,
  whitespace-collapsed and truncated to a bounded length (default 200 chars, with an
  ellipsis when truncated);
- `verified` — a boolean, true when an active (non-draft) `TestCase` verifies it
  (reusing the coverage notion, not a new computation).

The row **shall not** carry the full frontmatter or body (that is `export` / `show`);
the point is ~30 tokens per requirement.

## Output

- The default output **shall** be **NDJSON**: one compact JSON object per line, in
  qualified-name order, streamed so a huge model does not buffer in memory.
- With **`--json`**, the output **shall** be one document `{ "total": <n>, "offset":
  <k>, "rows": [ … ] }` where `total` is the count **before** paging.
- `--limit <N>` / `--offset <N>` **shall** page the rows (cursor paging); `--limit`
  bounds the emitted rows, `--offset` skips the first N. Without `--limit` all rows in
  scope are emitted.

## Scoping

`--where <predicate>` (repeatable, AND), `--status <s>`, `--tag <t>` and `--config
<CONF|features>` **shall** restrict the requirement set exactly as `stats` does —
reusing the same custom-field predicate ([[REQ-TRS-CFLD-001]]), tag/status filters and
the projection lens ([[REQ-TRS-PROJ-001]]) — so the same narrowing an LLM applied to
`stats` yields the matching `digest` slice. An unresolvable `--config` is a usage error
(exit `1`).

## LLM exposure

The digest **shall** be reachable by an MCP client as a **first-class read-only tool
`digest`** (mirroring the CLI, accepting `limit`/`offset`/`where`/`status`/`tag`/`config`
and returning the `{ total, offset, rows }` document), and via the `run_report`
allow-list.

**Source:** user request — Tier B of LLM-scale corpus scanning (token-budgeted bulk
requirement export). Read-only; aggregates existing data; no new element types or rules.

**Acceptance criteria:**

- `syscribe -m <root> digest` prints one NDJSON line per native `Requirement`; each line
  is valid JSON carrying `id`, `name`, `status`, `reqDomain`, `text` and `verified`.
- The `text` field is a single line (no embedded newline) bounded in length.
- `digest --json` emits one document with `total`, `offset` and a `rows` array; `total`
  is the pre-paging count.
- `digest --limit 5 --offset 5` emits at most 5 rows starting at the 6th, and `total`
  still reflects the full in-scope count.
- `digest --status approved` restricts the rows (and `total`) to approved requirements;
  `digest --where custom.k=v` and `digest --tag t` likewise restrict.
- `digest --config <C>` emits only requirements active in that variant; an unresolvable
  `--config` exits `1`.
- The MCP `digest` tool returns the same rows as `digest --json` for the same arguments
  and is advertised as read-only.
