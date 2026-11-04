# Incremental revalidation — measure first (GH #429)

**Status: measured; not built.** `REQ-TRS-TIMING-001`.

The issue asks for revalidating only the subgraph an edit touches, so that typing stays responsive on large product
lines, and says to *measure first and set a target before building*. This note records the measurement, the target and
the decision. The tool is `syscribe timing` (also `syscribe help timing`); a guard and a table generator are in
`crates/syscribe/tests/perf_budget.rs`.

## What happens on an edit

The language server reloads the whole model on `didSave` and `didChangeWatchedFiles` and revalidates it; it never
reloads on `didChange`, so a keystroke costs nothing. What a user waits for is **save → diagnostics**, which is *load
+ validate*. The MCP server behaves the same way after a write or a file-watch event. `validate --all-configs` and
`health` also validate each Configuration's projected variant; that is not on the save path (`health` caches it).

## Target

Save-to-diagnostics of **250 ms or less** (load + validate, release build) — the point at which a refresh stops
feeling instantaneous. A debug build is several times slower and is not the reference.

## Measurements (release build, one machine)

The repository's own models:

| model | files | elements | load ms | validate ms | variants ms | peak MiB |
|---|---:|---:|---:|---:|---:|---:|
| `model/` (this project) | 1,500 | 1,499 | 36 | 43 | 141 | 42 |
| `model_gti/` | 292 | 341 | 9 | 39 | 144 | 34 |

A synthetic model (`n` requirements, each with a test case; four Configurations; every requirement gated by a
feature), from `cargo test --test perf_budget -- --ignored --nocapture`:

| requirements + tests | files | elements | load ms | validate ms | variants ms | peak MiB |
|---:|---:|---:|---:|---:|---:|---:|
| 500 | 516 | 516 | 8 | 5 | 18 | 24 |
| 1,000 | 1,021 | 1,021 | 18 | 11 | 45 | 29 |
| 2,000 | 2,031 | 2,031 | 37 | 28 | 135 | 38 |
| 4,000 | 4,051 | 4,051 | 71 | 90 | 455 | 57 |
| 8,000 | 8,091 | 8,091 | 152 | 294 | 1,656 | 93 |

## What the numbers say

- **The largest real model is at 79 ms** of load + validate, a third of the target. The target is met up to about
  4,000 elements (161 ms) and first exceeded at 8,000 (446 ms).
- **Load is linear** (16× the elements, 19× the time). **Validation is not**: 59× for 16× the elements (about
  n^1.5), and the per-Configuration variants about n^1.6. Memory is linear and small (93 MiB at 8,000 elements).
- So the cost that would matter first is **validation growing faster than the model**, not re-doing unchanged work.

## Decision

Do **not** build per-package caching of the resolver and findings yet.

1. No model we can measure needs it: the largest is well inside the target, and the save-to-diagnostics path is not
   the variants path.
2. The validator's rules are mostly global (reverse indices, cross-package references, feature-model constraints,
   suspect links). An incremental scheme must know, for every rule, which edits can change its findings; a missed
   dependency yields a stale diagnostic, which is worse than a slow one. That is a large, risky change to buy
   nothing measurable today.
3. The super-linear growth is a cheaper lever. If a model approaches the target, **profile validation first** and
   remove the quadratic rule or rules (a scan of all elements per element); that restores linear time with no
   consistency risk.

## Trigger

Build incremental revalidation when `syscribe timing` reports **load + validate above 250 ms in a release build** on a
real model *after* profiling has removed the avoidable super-linear work — about 6,000 elements on the synthetic
model above. `syscribe timing` on the reader's own model is the check.

## If it is built

- Keep the equivalence the issue asks for as the acceptance test: for a sequence of random edits to a fixture model,
  the incrementally maintained findings must equal a full validation after each edit (a property test comparing the
  two). Until an incremental path exists there is nothing to compare, so that test is not written yet.
- Key the cache on content hashes of files; invalidate a rule's findings by the reverse index of what it reads.
- Start with the rules that are purely per-element (field and id checks), which are most of the rule count and none of
  the dependency risk, and leave the global rules on the full path.

## Guard

`perf_budget.rs` asserts generous ceilings on the repository's models (debug build) and that doubling a synthetic
model does not more than triple load + validation, so a change that makes either super-linear fails the suite.
