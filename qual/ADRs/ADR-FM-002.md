---
type: ADR
id: ADR-FM-002
name: "Use batsat as the sole SAT engine for feature-model analysis"
status: accepted
---

## Context

[[ADR-FM-001]] adopted an **in-tree DPLL** SAT solver for `feature-check --deep`, chosen for a
zero-dependency qualification story. In review we reconsidered: a hand-rolled solver is naive
(exponential, no clause learning), caps out at small models, and — however well unit-tested — carries
less correctness evidence than a mature, widely-used solver. The directive was to **use a real solver,
not maintain our own.**

We evaluated the mature pure-Rust options:

- **splr** (CDCL, MPL-2.0) — modern and fast, but **dropped incremental/assumption solving** (README:
  removed at 0.18), exposes **no library-level unsat cores**, and emits DRAT only via CLI/file. That
  forecloses the capabilities we want (selector cores, configurator, counting).
- **batsat** (MiniSat-derived CDCL, **MIT**) — incremental, `solve_limited(&[Lit])` under assumptions,
  **assumption-based unsat cores** (`unsat_core() -> &[Lit]`), in-library **DRAT** proofs, and an
  extensible **theory** API (a future path to the deferred numeric/parameter layer).

## Decision

Use **batsat (pinned `=0.6.0`, MIT)** as the **only** SAT solver. Remove the in-tree DPLL entirely.

- A small neutral clause IR (`solver::Cnf`/`Lit`) is built by the encoder ([[REQ-TRS-FMA-001]]) and
  handed to batsat; one `Solver` is primed from the encoding and reused across the many assumption
  queries (incremental).
- **Determinism** (REQ-TRS-FMA-006) holds: the deep analysis consumes only SAT/UNSAT *verdicts* (never
  witness models), and verdicts are canonical; our own enumeration/output order is fixed.
- Correctness of the integration is guarded by an exhaustive **brute-force truth-table oracle** in
  tests (a verifier, not a solver) run against batsat on 5,000 random formulas, plus pigeonhole-UNSAT
  and edge cases. This is *not* a second solver — it is how we keep the dependency honest.
- The new dependency is **MIT**, pure-Rust, in-process, no native/system library, no network — so the
  reproducible-build/qualification posture from ADR-FM-001 is preserved (one vendored crate replaces
  ~200 lines of bespoke solver).

## Consequences

- **+** CDCL scale and proven-in-use correctness; the in-tree solver is gone (less to maintain/qualify
  as bespoke code).
- **+** Unlocks the planned capabilities — selector-based cores, assisted configuration, variant
  counting, diagnoses, and in-library DRAT proofs — that splr could not provide.
- **+** batsat's theory API is a future avenue for the deferred numeric/parameter (SMT) layer.
- **−** One third-party dependency (batsat + `bit-vec`) now in the trust boundary; mitigated by pinning,
  MIT licensing, and the differential oracle tests.
- **−** Worst-case exponential still applies; the feature-count size guard (REQ-TRS-FMA-006) remains.
