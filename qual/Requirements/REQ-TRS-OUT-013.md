---
id: REQ-TRS-OUT-013
type: Requirement
title: Tool shall provide a read-only safety-readiness audit dashboard with a configurable PASS/FAIL verdict
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a read-only `audit` subcommand
(`syscribe -m <root> audit [--json] [--profile <name>] [--config <C>] [--all-configs]`)
that aggregates existing model data into a single safety-readiness dashboard plus a
configurable PASS/FAIL verdict. The command **shall reuse** existing validation,
coverage and profile logic and **shall not** reimplement validation or coverage.

This requirement refines the structured-output and exit-code contract of
REQ-TRS-OUT-006, reuses the named severity profiles of REQ-TRS-OUT-012, and honours
the configuration-projection lens of REQ-TRS-PROJ-001/002.

## Configuration projection (the `--config` lens)

`audit` **shall** honour the same configuration lens as `validate --config`
(GH #35): given `--config <CONF|features>`, the **entire** dashboard — the verdict
(W306 etc.), the orphan sets, and the coverage section — **shall** be computed only
over the elements **active** in that configuration (per `appliesWhen`, via
`projection::project`). A requirement that is inactive in the selected variant
**shall not** contribute to W306 or the orphan counts. With no `--config`, behaviour
is unchanged (whole-model view). An unresolvable `--config` argument is a usage error
(exit `1`); a `--config` with no feature model present falls back to the whole-model
audit.

`audit --all-configs` **shall** audit every stored `Configuration`'s projected
variant and print a per-configuration PASS/FAIL summary, exiting non-zero if **any**
variant fails (the product-line CI gate, mirroring `validate --all-configs`).

The projected verdict **shall** be computed via the **projection-aware** validation
path (`projection::validate_projected`), exactly as `validate --config` does, so that
`audit --config <C>` and `validate --config <C>` **agree on the error count** for the
same variant (GH #36). In particular a `TestCase` that survives the projection but
whose `verifies:` target was projected **out** of the variant **shall not** be
mis-counted as a dangling `TestCase` nor surface a phantom Error finding: dangling
detection considers only the **active** `TestCase`s but resolves their references
against the **full** model.

## Dashboard sections

The text report **shall** present, and `--json` **shall** mirror, the following
sections:

1. **Requirement status split** — counts of native `Requirement` elements by
   `status:` (draft / review / approved / implemented / verified), reported
   **overall** and **per top-level package** (the first `::` segment of the
   qualified name / the top-level directory under the model root).
2. **SIL / ASIL distribution** — counts of native `Requirement` elements by
   `silLevel` and by `asilLevel`, including a `QM/none` bucket for requirements
   that declare neither.
3. **Per-configuration coverage %** — `covered` / `gap` / `N-A` per
   `Configuration` and overall, computed by **reusing the `matrix` coverage
   computation** (one definition shared with `matrix`; no duplicated coverage
   logic). When no feature model is present, the flat requirement→TestCase
   coverage view **shall** be shown instead.
4. **Orphans** — counts and ids of:
   - native `Requirement`s with no active verifying `TestCase`;
   - native `Requirement`s that no element `satisfies:`;
   - `TestCase`s whose `verifies:` is empty or resolves to nothing (dangling);
   - `Requirement`s with neither `derivedFrom` nor `derivedChildren`.

   A **parent** `Requirement` (one that has `derivedChildren` — i.e. other
   requirements `derivedFrom` it) **shall be excluded** from both the
   `unsatisfiedRequirements` and the `unverifiedRequirements` orphan sets (GH
   #37). A parent is satisfied and verified only **transitively** through its
   leaves; §12.4 / `E312` forbid a parent from appearing in any `satisfies:`
   list and W002/W300/W306 already suppress parents, so flagging it here would
   always be a false positive. Genuine satisfaction/verification gaps still
   surface on the **leaf** requirements. The `untraced` set (requirements with
   neither `derivedFrom` nor `derivedChildren`) is unaffected.
5. **Readiness verdict** — a single PASS/FAIL line that names **why** it failed
   (the triggering codes / counts).

## Policy and exit code

The tool **shall** run validation via `validator::validate_with_config` and
compute the verdict as follows. The audit **FAILS** (exit code `2`) when **any**
of the following hold:

- any **Error**-severity finding exists; or
- (default policy) any **W306** finding (the unsatisfied-safety-mechanism gate)
  exists; or
- when `--profile <name>` is given, any finding **promoted** by that
  `[profiles.<name>]` policy in `<model_root>/.syscribe.toml` is present (reusing
  the REQ-TRS-OUT-012 / issue #18 profile loader and promotion logic — not a
  reimplementation).

Otherwise the audit **PASSES** (exit code `0`). An undefined `--profile` name (or
a missing `.syscribe.toml`) is a usage error and **shall** exit `1`.

## JSON output

With `--json`, the whole rollup **shall** be emitted as one document with the
shape:

```json
{
  "statusSplit": { "...": "..." },
  "integrityDistribution": { "...": "..." },
  "coverage": { "...": "..." },
  "orphans": { "...": "..." },
  "verdict": { "pass": true, "reasons": [] }
}
```

**Source:** Issue #15 (safety-readiness dashboard); refines REQ-TRS-OUT-006;
reuses REQ-TRS-OUT-012.

**Acceptance criteria:**

- `syscribe -m <root> audit` prints the five sections and a PASS/FAIL verdict.
- A "ready" model (approved requirements, satisfied and covered, no W306, no
  errors) audits PASS and exits `0`.
- A "not-ready" model (a SIL-4 / ASIL-D requirement that is draft or
  unsatisfied, tripping W306) audits FAIL, names W306 in the verdict, and exits
  `2`.
- `audit --json` emits one document carrying `statusSplit`, `integrityDistribution`,
  `coverage`, `orphans` and `verdict`, and is valid JSON.
- The per-configuration coverage section reuses the `matrix` coverage
  computation, and the existing `matrix` tests continue to pass unchanged.
- A model with a high-integrity requirement that is `appliesWhen`-gated out of a
  configuration audits **FAIL** whole-model but **PASS** under `audit --config <that
  config>` (the requirement is projected out); `audit --all-configs` reports each
  Configuration's verdict and exits non-zero if any fails.
- For a variant where a `TestCase` verifies a requirement that is projected out,
  `audit --config <C>` reports `danglingTestCases.count == 0` and a `verdict.pass`
  consistent with `validate --config <C>` being clean (GH #36 — no phantom finding).
- A **parent** requirement (one with `derivedChildren`) whose leaves are each
  satisfied and verified is **absent** from `orphans.unsatisfiedRequirements`
  and `orphans.unverifiedRequirements` (GH #37), consistent with `validate` not
  emitting W002/W300 for that parent. Genuine gaps still surface on the leaves.
