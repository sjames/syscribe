---
id: REQ-TRS-OUT-021
type: Requirement
name: Tool shall provide a read-only stats command that aggregates the model into facet histograms for fast LLM corpus scanning
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `stats` subcommand
(`syscribe -m <root> stats [--json] [--group-by <facet>] [--where <predicate>] [--status <s>] [--tag <t>] [--config <CONF>]`)
that aggregates existing model data into a single compact **corpus-shape digest** —
per-facet histograms plus coverage and orphan rollups — so that a large-language-model
client can grasp the shape of a model with **tens of thousands** of `Requirement`
elements in **one** call, without reading individual element files. The command
**shall reuse** existing validation, coverage and matrix logic and **shall not**
reimplement validation, coverage or reverse-index computation.

This requirement is the "metadata rollup and faceting" capability (Tier A) of the
LLM-scale scanning work. It complements the audit dashboard of [[REQ-TRS-OUT-013]]
(which is a fixed safety-readiness verdict) by offering a **general, pivotable**
histogram surface primarily intended for machine consumption.

## Facets

`stats` **shall** compute, over the native `Requirement` elements in scope, a
histogram (a `value → count` map) for each of the following facets, plus the total
count:

1. **`status`** — count by `status:` (draft / review / approved / implemented /
   verified), with a bucket for requirements declaring none.
2. **`reqDomain`** — count by `reqDomain:` (system / hardware / software), with an
   `unset` bucket.
3. **`silLevel`** and **`asilLevel`** — count by integrity level, each including a
   `QM/none` bucket for requirements declaring neither (consistent with the
   REQ-TRS-OUT-013 §3.2 integrity distribution).
4. **`package`** — count by **top-level package** (the first `::` segment of the
   qualified name / the top-level directory under the model root). The map **shall**
   report the top-N packages by count plus an aggregated `other` bucket when the
   package count exceeds N (default N = 20).
5. **`tags`** — a tag histogram over `tags:` (a requirement contributes to every
   tag it carries).

## Coverage and orphan rollup

In addition to the facet histograms, `stats` **shall** emit:

- a **coverage** rollup — `{ verified, unverifiedLeaves, parentsMissingIntegration }`
  computed by **reusing** `coverage::coverage_summary` (the same numbers `matrix` and
  the coverage command report; no duplicated coverage logic); and
- an **orphans** rollup — counts of native `Requirement`s with no active verifying
  `TestCase`, with no element `satisfies:`-ing them, and with neither `derivedFrom`
  nor `derivedChildren` (the "untraced" set). A **parent** `Requirement` (one with
  `derivedChildren`) **shall be excluded** from the unsatisfied and unverified orphan
  sets, exactly as REQ-TRS-OUT-013 §4 / GH #37 specify, so the two commands agree.

The orphan and coverage rollups **shall** be derived from the validator reverse
indices (`verified_by`, `derived_children`, …) already exposed on the validation
result — `stats` **shall not** recompute traceability.

## Pivoting and scoping

- **`--group-by <facet>`** **shall** re-key the primary histogram by the named facet
  crossed with `package` — i.e. it **shall** emit, per top-level package, the chosen
  facet's histogram (e.g. `--group-by status` yields a status split per package,
  matching REQ-TRS-OUT-013 §3.1's "overall and per top-level package"). An
  unrecognised facet name is a usage error (exit `1`).
- **`--where <predicate>`**, **`--status <s>`** and **`--tag <t>`** **shall** restrict
  the aggregated element set before counting, reusing the **existing** custom-field
  `--where` predicate (REQ-TRS-CFLD) and the existing tag/status filters — `stats`
  **shall not** introduce a new filter grammar.
- **`--config <CONF|features>`** **shall** honour the same configuration-projection
  lens as `audit --config` / `validate --config`: every facet, the coverage rollup
  and the orphan rollup **shall** be computed only over the elements **active** in
  that variant (via `projection::project`). With no `--config`, the whole model is
  aggregated. An unresolvable `--config` argument is a usage error (exit `1`); a
  `--config` with no feature model present falls back to the whole-model view.

## Output

The default (text) output **shall** be a compact human-readable digest: the total,
each facet as a small sorted table, and the coverage/orphan rollup lines — an
executive "briefing" of the corpus.

With **`--json`**, the whole rollup **shall** be emitted as one document with the
shape:

```json
{
  "total": 15042,
  "facets": {
    "status":    { "approved": 8100, "draft": 3200, "implemented": 3742 },
    "reqDomain": { "software": 9000, "hardware": 4000, "system": 2042 },
    "silLevel":  { "4": 300, "QM/none": 12342 },
    "asilLevel": { "D": 300, "QM/none": 12342 },
    "package":   { "VehicleSystem::Powertrain": 812, "other": 5000 },
    "tags":      { "safety": 2000, "perf": 800 }
  },
  "coverage": { "verified": 11000, "unverifiedLeaves": 900, "parentsMissingIntegration": 42 },
  "orphans":  { "unverifiedRequirements": 900, "unsatisfiedRequirements": 300, "untraced": 120 }
}
```

`--group-by <facet>` **shall** replace the flat `facets.<facet>` map with a nested
`byPackage: { "<pkg>": { "<value>": <count> } }` map for the chosen facet.

## LLM exposure

The aggregation **shall** be reachable by an MCP client in two ways, so the digest is
the LLM's cheap first hop before any per-element `get_element`:

- as a **first-class read-only MCP tool `stats`** (a `read_only_hint` tool mirroring
  `stats --json`, alongside `coverage` and `coverage_matrix`), accepting the same
  `group_by` / `where` / `status` / `tag` / `config` arguments; and
- via the read-only report allow-list, so `run_report` can invoke it.

**Source:** user request — Tier A of LLM-scale corpus scanning (fast metadata rollup
and faceting for models with ~15 000 requirements). Read-only; aggregates existing
data; no new element types or validation rules. Reuses REQ-TRS-OUT-013 coverage/orphan
definitions, the REQ-TRS-CFLD `--where` predicate, and the REQ-TRS-PROJ `--config` lens.

**Acceptance criteria:**

- `syscribe -m <root> stats` prints the total, each facet table, and the
  coverage/orphan rollup lines.
- `stats --json` emits **one** valid JSON document carrying `total`, `facets`
  (`status`, `reqDomain`, `silLevel`, `asilLevel`, `package`, `tags`), `coverage` and
  `orphans`.
- The `coverage` numbers equal those reported by the coverage/`matrix` computation
  for the same model (shared definition — a divergence is a test failure).
- A **parent** requirement (one with `derivedChildren`) is **absent** from
  `orphans.unsatisfiedRequirements` and `orphans.unverifiedRequirements`, consistent
  with REQ-TRS-OUT-013 / `validate` not flagging it.
- `stats --group-by status` emits a per-top-level-package status histogram
  (`byPackage`), and an unknown `--group-by` facet exits `1`.
- `stats --where custom.foo=bar` and `stats --status approved` each restrict the
  aggregated set, and the reported `total` reflects the filtered count.
- `stats --config <C>` aggregates only the elements active in that variant; a
  requirement `appliesWhen`-gated out of `<C>` does not contribute to any facet,
  coverage or orphan count. An unresolvable `--config` exits `1`.
- The MCP `stats` tool returns the same document as `stats --json` for the same
  arguments and is advertised as read-only.
