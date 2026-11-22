---
type: ADR
id: ADR-SYS-COVERAGE-002
name: "Coverage axes are shared across plans by a named ConditionSet, not copied inline"
status: proposed
---

## Context

ADR-SYS-COVERAGE-001 declares coverage axes inline on each `TestPlan`
(`conditions:`) and says to promote them to an element "only if one envelope is
reused across many plans or requirements". Slices 1–2b implement the inline
form. The same envelope (ambient, supply voltage, firmware variant) is now
expected on several plans: a bench plan, a chamber plan and a field plan verify
the same requirements over the same operating envelope. Copying the axes means
a change to the envelope (a new hot point) must be made in every plan, and a
missed plan silently keeps the smaller envelope and reports `pass`.

## Decision

1. **Inline stays.** A plan with its own `conditions:` is unchanged and needs
   no other element. Nothing existing is migrated.
2. **A plan may instead reference shared axes.** `conditionSet: <ref>` names a
   `ConditionSet` element (`CS-*`, id-identified) that carries the same
   `conditions:` shape as the inline form, with `minUnits`, `mode` and `points`
   optional defaults.
3. **Plans override, never merge axes.** A plan that sets both `conditionSet:`
   and its own `conditions:` is an error: one source of axes per plan. The
   plan's own `minUnits`, `mode` and `points` win over the set's defaults, so a
   field plan can require more units without redefining the axes.
4. **The set is normative content.** Changing it changes the required coverage
   of every plan that references it. Plans use the ordinary suspect-link
   baseline (`traceBaselines:`), so W090 flags each plan for review, and
   results stay valid because the TestCase is untouched (ADR-001 consequence).
5. **Resolution happens before judging.** The reference is resolved into the
   same `Coverage` value inline axes produce; `plan_gaps`, records, verdicts,
   `trace` and `audit` are unchanged and see no difference.
6. **Codes.** An unresolved reference or a non-`ConditionSet` target is an
   error; a malformed set reuses E607/E608 against the set; a `ConditionSet`
   no plan references is an advisory warning.

## Alternatives considered

- **`extends:` between plans.** Couples plans to each other and mixes scope
  (members, rigs) with the envelope; a plan inheriting from a plan it does not
  otherwise relate to is hard to review.
- **Project-wide axes in `.syscribe.toml`.** Not traceable or reviewable per
  requirement, and not versioned with the model's baselines.
- **Keep copying.** Rejected: the failure mode is a silently smaller envelope.

## Consequences

- One edit widens the envelope for every plan that uses it; `suspect list`
  shows which plans need review.
- A new element type, id prefix, template and help topic (discoverability
  checklist) and validation codes in both catalogues.
- Requirement-level display (`trace`, worst over plans) is unaffected.
- Decide before implementing: whether `ConditionSet` may name `unit` classes
  (`minUnits` per set) or leaves units on the plan; this ADR leaves them as
  plan-overridable defaults.
