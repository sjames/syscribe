# Test Plans

A **TestPlan** groups reusable `TestCase`s into the unit a team actually executes,
reviews, and reports against — a smoke plan, an integration plan, a certification
campaign. `TestCase`s stay reusable atoms; a `TestPlan` is a curated, per-product
artifact.

The two levels are distinct:

- a **TestCase** is written once and may belong to many plans and run in many products
  (which products it runs in is computed from its own `appliesWhen:`);
- a **TestPlan** is authored for one *or more* `Configuration`s at one *scope*. A product
  typically has several plans (unit / smoke / integration / …), and a plan may be reused
  across several products.

## Declaring a plan

A `TestPlan` (`type: TestPlan`, stable `TP-*` id) lives by convention under a
`TestPlans/` area:

```yaml
---
type: TestPlan
id: TP-DELIVERY-INTEGRATION-001
name: "UAV delivery — integration plan"
status: approved
scope: integration
configurations: [CONF-UAV-DELIVERY-001, CONF-UAV-SURVEY-001]   # reused across two products
demonstrates: [SG-UAV-001]                                     # optional safety-case leg
testCases: [TC-UAV-NAV-001, TC-UAV-SAFE-003]
selection:
  testLevels: [L3, L4]
  tags: [integration]
---
Integration plan executed before each delivery/survey release.
```

| Field | Meaning |
|---|---|
| `scope` | `unit` · `smoke` · `integration` · `hil` · `certification` · `security` · `regression` (other values warn `W610`). Distinguishes multiple plans over the same config. |
| `configurations` | A `Configuration` id or list — the product variant(s) the plan is for. **Omit** for a config-agnostic plan that applies everywhere. |
| `demonstrates` | Optional. Goals / requirements the plan is offered as evidence for (a safety-case leg). Not required. |
| `testCases` | Explicit members. |
| `selection` | An **additive** query (`testLevels`, `domains`, `tags`) unioned with `testCases`. A `selection` with no sub-fields matches nothing; draft TestCases are not swept in. |

**Effective members** = `testCases` ∪ `selection` matches (deduped). Membership of a given
configuration is still *computed* from each member's own `appliesWhen:` — the plan only
declares *which products it is a plan for*. A member active in **none** of the plan's
configurations is an escaping member (`W611`).

## Organising plans

- **Multiple plans per product** — write one file per `(configuration, scope)`:
  `TP-DELIVERY-UNIT-001`, `TP-DELIVERY-SMOKE-001`, `TP-DELIVERY-INTEGRATION-001`, all with
  `configurations: [CONF-UAV-DELIVERY-001]` and distinct `scope`.
- **Reuse a plan across products** — list several configs in `configurations:`.
- **Config-agnostic plan** — omit `configurations:`.

## Working with plans

```bash
syscribe -m model/ testplan                       # list: scope, configs, coverage %, verdict
syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001        # detail
syscribe -m model/ testplan TP-DELIVERY-INTEGRATION-001 --json # machine-readable
```

The detail view shows the resolved members (flagging escaping ones), the **in-scope
requirements** (the goal-closure of `demonstrates:`, or the `verifies:` targets of the
members when `demonstrates:` is absent), a per-configuration coverage grid, and a
rolled-up verdict (`pass` · `fail` · `incomplete` · `empty`) folded from ingested results.

The **`--plan TP-X` lens** scopes other reports to a plan and composes with `--config`:

```bash
syscribe -m model/ matrix --plan TP-DELIVERY-INTEGRATION-001
syscribe -m model/ verification-depth --plan TP-DELIVERY-INTEGRATION-001 --sil 4
syscribe -m model/ audit --plan TP-DELIVERY-INTEGRATION-001        # readiness scoped to the plan
```

For `audit`, the lens validates the **whole** model (so a reference escaping the plan
subset is never mistaken for a defect) and counts only findings on the plan's in-scope
elements toward the verdict.

## Validation

`E600`–`E606` (malformed id, unresolvable member / config / demonstrates target, bad
selection levels/domains, bad status) and `W610`–`W616` (non-recommended scope, escaping
member, empty plan, pinned draft member, demonstration gap, results-gated failing member,
duplicate `(configurations, scope)`) — full list in the
[Rule Reference](../validation/rules.md#testplan-e600e606-w610w616). A duplicate `id` is the
generic `E101`.
