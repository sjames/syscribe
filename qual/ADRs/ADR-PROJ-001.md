---
type: ADR
id: ADR-PROJ-001
name: "Configuration projection: a filter-and-reuse --config lens, per-variant validation, and a global SAT guarantee"
status: proposed
---

## Context

With the feature model, `appliesWhen` conditioning, and `Configuration` selections in place, the repository
is a **150% model** of the whole product line. We need to (a) *view* and *validate* the model **through the
lens of one configuration** — the **100% model** of a single product — and (b) guarantee the variability is
internally consistent (references don't break in some variant).

The projection predicate already exists and is exercised by `matrix` and `W015`: an element is **active** in a
selection iff its `appliesWhen` holds (no `appliesWhen` ⇒ always active). The open questions were how to
expose it, how strict to be about references that "escape" a variant, and whether to reason per-configuration
or across all valid configurations.

Throughout, the **opt-in invariant** holds: a model with no `FeatureDef` is dormant and behaves exactly as
before.

## Decision

1. **A composable `--config` lens flag** (not parallel commands) on the read/validation commands
   (`validate`, `trace`, `why`, `who-verifies`, `list`, `refs`, `links`, `export`, `diagram`). Its argument is
   a stored `Configuration` (id/qname) **or** an ad-hoc feature set (what-if) — [[REQ-TRS-PROJ-001]].

2. **Filter-and-reuse.** A projection is itself a valid Syscribe model, so the lens is implemented by
   filtering to the active subset and running the *existing* command/validator over it. Minimal new code;
   the per-variant rules (coverage, §12, cross-refs) come for free — [[REQ-TRS-PROJ-002]].

3. **`validate --config` is full re-validation in the lens** — *certify the variant*, not just the superset.

4. **Escaping references are classified by kind**: structural/typing escapes are **errors** (`E226`),
   traceability escapes are **warnings** (`W019`); `appliesWhen` operands are excluded —
   [[REQ-TRS-PROJ-003]].

5. **A config-independent global guarantee** in `feature-check --deep`: prove `appliesWhen(X) ⇒ appliesWhen(Y)`
   for each reference edge over *all* valid configurations via SAT (`Φ ∧ aw(X) ∧ ¬aw(Y)` unsatisfiable),
   structural ⇒ `E227` (error), traceability ⇒ `W020` (warning), each with a witness selection —
   [[REQ-TRS-PROJ-004]].

6. **Family-level checks**: `validate --all-configs` gate, dead elements (`W021`), aggregate coverage
   (`W022`), and `diff --config A --config B` — [[REQ-TRS-PROJ-005]].

7. **Scope**: single-level selection (two-level resolution deferred); read-time projection (no on-disk
   materialization beyond `export --config`); Boolean layer for the global/SAT parts (reusing the
   [[ADR-FM-002]] engine).

## Alternatives considered

- **Dedicated `project`/`view` command** instead of a flag — rejected: less composable, duplicates each
  view's logic. The flag turns every existing view into a lens.
- **Materialize the 100% model to disk** by default — rejected: keep the 150% model the single source of
  truth; `export --config` covers handoff on demand.
- **Uniform-strict escaping** (every active→inactive reference is an error) — rejected: over-flags legitimate
  traceability to broader/cross-cutting elements. Chose the by-kind policy.
- **Per-config detection only** (no global proof) — rejected: a structural escape would surface only if you
  happen to author/view the exposing configuration. The SAT guarantee covers all variants.
- **A separate validator for the lens** — rejected in favour of filter-and-reuse.

## Consequences

- **+** Per-variant certification — the qualification story is "this product's requirements are covered and
  satisfied by this product's tests and architecture", not the superset's.
- **+** The global guarantee catches latent variability bugs (broken references) in variants nobody authored.
- **+** Small implementation: the lens is a filter over the existing stack; only escaping-ref classification,
  the global SAT rule, and the family checks are new logic.
- **+** Composable and dormant-safe (no `FeatureDef` ⇒ inert).
- **−** `validate --all-configs` is O(configurations × validation); fine for typical counts, may need batching
  for very large families.
- **−** The structural-vs-traceability taxonomy is a policy that may need tuning as new reference fields are
  added.
- **−** New finding codes `E226`/`E227`/`W019`–`W022`; two-level resolution, DRAT, and on-disk materialization
  remain out of scope.
