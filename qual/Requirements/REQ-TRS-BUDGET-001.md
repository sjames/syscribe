---
id: REQ-TRS-BUDGET-001
type: Requirement
name: Tool shall evaluate budget expressions on CalculationDef (bodyLanguage budget) — E866/E867/E868/W060
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** support a restricted arithmetic **budget expression language** for a
`CalculationDef` whose `bodyLanguage:` is `budget`, enabling scalar engineering budgets
(mass, power, timing, link margin) to be evaluated and bound-checked without a full KerML
expression evaluator (§22.2).

### Grammar

```
expr        ::= term (('+' | '-') term)*
term        ::= factor (('*' | '/') factor)*
factor      ::= NUMBER | feature_ref | '(' expr ')'
feature_ref ::= IDENT ('::' IDENT)*
```

The expression is read from the element's `body:`. A `feature_ref` operand **shall** resolve,
in order, to: (1) an inline `features:` entry of the `CalculationDef` itself whose `value:`
or `default:` is numeric (bare names — "in scope"); (2) an element resolved by the full
qualified name carrying a numeric `value:`/`default:`; or (3) the owning element of a
`<owner>::<feature>` reference, reading that feature's `value:`/`default:`.

### New field — `evaluate`

A `CalculationDef` **may** declare `evaluate: <qn>`, the qualified name of a `ConstraintDef`
that bounds the budget result.

### Validation rules

| Code | Condition |
|---|---|
| `E866` | `evaluate:` is set but does not resolve to a `ConstraintDef` (drafted as `E800`, already in use). |
| `E867` | The `bodyLanguage: budget` `body:` expression has a syntax error (does not parse against the grammar). |
| `E868` | A `feature_ref` operand in the budget expression resolves to no numeric attribute in scope. |
| `W060` | The budget evaluates to a value that violates the `evaluate:` constraint (best-effort, for a constraint reducible to `<lhs> <op> <number>`). Draft-suppressed; gateable with `--deny W060`. |

`E866`–`E868` are errors; `W060` is a warning. The checks **shall** apply only to
`CalculationDef` elements with `bodyLanguage: budget`.

> **Code note:** the format spec drafted these as `E800`/`E801`/`E802`, but those are already
> in use (HazardousEvent). They are reassigned to **`E866`/`E867`/`E868`**; `W060` is free
> and unchanged. The spec §22.2 is corrected.

**Source:** §22.2 (Budget Expression Language, extends §8.9), GH #67.

**Acceptance criteria:**

- A budget body `"A::x + B::y"` whose operands resolve evaluates to their numeric sum.
- A malformed body (e.g. `"A + + B"`) raises `E867`.
- A body referencing an attribute that resolves to no value raises `E868`.
- `evaluate:` pointing at a non-`ConstraintDef` raises `E866`.
- A budget whose value violates an `evaluate:` constraint of the form `<lhs> <op> <number>`
  raises `W060`; a value within bound does not.
- `W060` is draft-suppressed and gateable with `--deny W060`.
- Existing `CalculationDef` elements (which use `expression:`, not `bodyLanguage: budget`)
  are unaffected.
