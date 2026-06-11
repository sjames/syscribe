---
id: REQ-TRS-PARAM-002
type: Requirement
title: Tool shall evaluate parameterConstraints numeric expressions against each Configuration
status: draft
reqDomain: software
verificationMethod: test
---

A package (`_index.md`) may declare `parameterConstraints:` — a sequence of cross-feature numeric constraints coupling typed feature `parameters:` (§9.7) to one another or to literal bounds. Each entry carries an `expression:`, an optional `appliesWhen:` predicate, an optional `id:`, and an optional `severity:` (`error` (default) or `warning`). Today these are accepted and their parameter paths are resolved (`E213`) and their `appliesWhen:` features are checked for use (`W014`), but the **expression is never evaluated** against any `Configuration`, so a configuration that violates a declared coupling (e.g. an AMP topology bound to a single core) passes `feature-check` silently (GH #14).

When a feature model is present, the tool **shall** evaluate every `parameterConstraints` expression during `feature-check`:

| Code | Condition |
|---|---|
| `E221` | In some `Configuration` for which the constraint's `appliesWhen:` predicate holds, the `expression:` evaluates to **false** (the coupling is violated). Default severity. |
| `W025` | The same violation, when the constraint declares `severity: warning` (the violation is reported as a warning rather than an error). |

### Expression language

The tool **shall** evaluate an `expression:` of the form `LHS <op> RHS` where:

- `<op>` is one of `==`, `!=`, `>=`, `<=`, `>`, `<`;
- `LHS` and `RHS` are arithmetic expressions over numeric literals and **parameter references**, supporting `+`, `-`, `*`, `/`, and parentheses;
- a **parameter reference** uses the canonical dotted form `Features::Path::To::Feature.paramName` — `::` separates the feature's qualified-name segments, and a single `.` separates the parameter (member) from its owning feature. This same dotted form is the canonical syntax for `parameterBindings:` keys and `bindTo:` targets, so every reference to a feature parameter is written identically and the feature/parameter boundary is unambiguous. The legacy all-`::` member form (`Features::Feature::paramName`) is **not** accepted (a `parameterBindings` key in that form is malformed → `E222`). A `parameterConstraints` `expression:` that uses the `::`-member form for a parameter **shall** raise `E213` (with a hint to use the dotted form) — it **shall never** be silently dropped (GH #14 re-open).

### Command coverage

These binding and constraint checks **shall** be enforced by **both** `validate` and `feature-check`, so a product line checked holistically (`feature-check`) gets the same parameter-range (`E205`) and binding enforcement as the per-element `validate` pass — neither command may leave a declared range/constraint unenforced.

For a given `Configuration`, a parameter reference (`Features::….paramName`) **shall** resolve to: the value bound in that configuration's `parameterBindings:` under that same key, else the parameter's fixed `value:` / `default:`. If any referenced parameter cannot be resolved for a configuration (unbound with no default), the constraint **shall** be skipped for that configuration (no `E221`/`W025`) — the unbound-required case is already covered by `W017`. If a referenced path does not resolve to any declared parameter, `E213` is emitted (as today) and the expression is not evaluated.

### Compound `appliesWhen:` parsing

The constraint's `appliesWhen:` predicate **shall** be parsed with the same boolean grammar used elsewhere (`and` / `or` / `not` / parentheses; a bare qualified name or a list meaning AND), not treated as a single literal feature name. A constraint **shall** be evaluated only against configurations whose feature selections satisfy this predicate; a constraint with no `appliesWhen:` applies to every configuration. The `W014` "feature not selected in any Configuration" check **shall** likewise operate on the parsed operands of the predicate (fixing the case where `appliesWhen: A and B` was reported as one unknown feature `"A and B"`).

These checks are emitted only when at least one `FeatureDef` exists (variability is opt-in) and are dormant when no `parameterConstraints` are declared.

**Source:** §9.7, §9.11 (`E221`); GH #14.

**Acceptance criteria:**

1. A constraint `expression: "Features::Topology.maxCpus >= 2"` with `appliesWhen: Features::Amp` produces an `E221` for a configuration that selects `Amp` and binds `maxCpus = 1` (via `parameterBindings: { Features::Topology.maxCpus: 1 }`), and produces no finding for a configuration that binds `maxCpus = 2`.
2. A constraint with `severity: warning` produces `W025` (not `E221`) on violation, and `feature-check --deny W025` exits non-zero.
3. A constraint whose `appliesWhen:` does not hold for a configuration is not evaluated against it (no false `E221`).
4. A compound `appliesWhen: Features::CortexM33 and Features::Amp` is parsed as two features (the constraint applies only when both are selected) and does not raise a `W014` "unknown feature" for the whole string.
5. A constraint referencing an undeclared parameter path still raises `E213` and is not evaluated.
6. A constraint `expression:` written with the `::`-member form (`Features::Topology::maxCpus >= 2`) raises `E213` naming the dotted form, rather than silently passing (GH #14 re-open).
7. `feature-check` enforces parameter `range:` (`E205`) and the other binding rules, not only `validate`.
