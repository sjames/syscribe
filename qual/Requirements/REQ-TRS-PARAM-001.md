---
id: REQ-TRS-PARAM-001
type: Requirement
name: FeatureDef Parameter Binding Validation
title: Tool shall validate Configuration parameterBindings against FeatureDef parameter declarations
status: draft
reqDomain: software
verificationMethod: test
---

A `FeatureDef` may declare typed `parameters:` (§9.7) — quantitative variability bound to concrete values in a selecting `Configuration` via `parameterBindings:` (a map keyed by `<FeatureDef qualified name>.<parameter name>`, the canonical dotted parameter-reference form — `::` between feature segments, a single `.` before the parameter member; see [[REQ-TRS-PARAM-002]]). When a feature model is present, the tool **shall** enforce the following single-level binding rules:

| Code | Condition |
|---|---|
| `E203` | `parameterBindings` binds a parameter of a feature that is **not selected** in this `Configuration` |
| `E204` | `parameterBindings` binds a parameter that is **fixed** (`isFixed: true`, a `value:`, or a `derivedFrom:` expression) and may not be overridden |
| `E205` | A bound value falls **outside** the parameter's `range:` (`"min..max"`) constraint |
| `E206` | A bound value is **not a member** of the parameter's `enumValues:` set |
| `E222` | A `parameterBindings` key does not resolve to a declared parameter — malformed path (including the legacy all-`::` member form `Features::Feature::param`, which lacks the required `.`), unknown `FeatureDef`, or undeclared parameter name |
| `W017` | A selected feature declares a parameter `isRequired: true` (not fixed, no `default:`) that the `Configuration` does **not** bind |

These checks are emitted only when at least one `FeatureDef` exists (variability is opt-in). `W017` uses a fresh code because §9.11's nominal `W010` is already taken by test-result ingestion in this tool.

**Related (separate requirements):** two-level `bindTo:` parameter propagation (`E202`), circular `derivedFrom:` expression detection (`E207`), and cross-feature `parameterConstraints` path resolution (`E213`/`W014`) are enforced by `feature-check`. Numeric **evaluation** of `parameterConstraints` expressions (`E221`/`W025`) is specified by [[REQ-TRS-PARAM-002]]; inclusive `range:` syntax (`"min..=max"`) by [[REQ-TRS-PARAM-003]].

**Source:** §9.7, §9.11; follow-up to the variability feature set.

**Acceptance criteria:** For each of `E203`, `E204`, `E205`, `E206`, `E222`, and `W017`, a crafted `Configuration` triggering exactly that condition produces a finding with that code; a `Configuration` whose bindings are all valid (selected feature, configurable parameter, in-range, in-enum, all required parameters bound) produces none of these findings.
