---
type: Requirement
id: REQ-TRS-PARAM-001
name: "constraints check evaluates ConstraintDef expressions against the model's attribute values and Configuration parameters"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - execution
---

`syscribe constraints check` shall evaluate the model's constraints with the expression language of REQ-TRS-EXPR-001 against value bindings taken from the model (GH #274, part of #272).

## Behavior

- **What is checked.** Every `Constraint` usage (a top-level element or an entry of `features:` with `type: Constraint`) whose `typedBy:` resolves to a `ConstraintDef` with an `expression:`; and every entry of `requires:` and `assume:` on a `Requirement`/`RequirementDef`, either an inline `expression:` or a `typedBy:` constraint definition. `isNegated: true` inverts the result. An `assume:` clause is reported as an assumption.
- **Parameters.** The parameters of a `ConstraintDef` are its `features:` entries with `direction: in` (or `inout`). A usage binds them with `bindingConnections: [{left: <parameter>, right: <expression>}]`, where `right` is any expression of the language: an attribute path (`Pkg::Part::mass`), a literal or a calculation over several. A parameter with no binding takes its own `value:` (`valueKind: default-bound`), read with its `unit:`; one with neither is unresolved.
- **Attribute values.** The value of an inline `features:` entry with a `value:` is found as `<qualified name of the owner>::<feature>` (also `<owner>.<feature>`), and the `value:` of an `Attribute` element as its qualified name. A `unit:` (`SI::kg`, `kg`) scales and dimensions the number; a `typedBy: ISQ::…Value` without a unit means SI base units. Under `--config <C>` the model is projected onto the Configuration first and its `parameterBindings:` (else the parameter's fixed `value:`, else `default:`) are found as `<feature qualified name or FEAT id>.<parameter>`. `--set name=value` binds or overrides any name, last.
- **Typing at the binding.** A parameter typed `ScalarValues::*` (or with no type) receives the number in SI base units with its dimension dropped (a `unit: SI::kg` `4.2` is `4.2`, a `unit: SI::g` `500` is `0.5`); a parameter typed `ISQ::…Value` receives the dimensioned quantity, so a mismatch is a dimension error.
- **Result per check:** `pass`, `fail`, `unresolved` (one or more names have no value; they are listed) or `error` (parse, type, dimension or domain error, with the message). The text report is a table; `--json` is `{results:[{id, kind, constraint, expression, status, message, bindings, unresolved}], summary:{pass, fail, unresolved, error}}`.
- **Exit code.** `0` when nothing failed or errored; `2` when any check failed or errored, or — with `--strict` — is unresolved; `1` for a usage error (unknown option, unknown configuration).
- **`--all-configs`** evaluates every check once per Configuration and reports a check × configuration table (`--json`: `{configurations:[{id, summary, results}]}`), exiting `2` if any configuration fails.
- The command is read-only and deterministic.

## Calculations and sweeps

- **`constraints calc <CalculationDef> [--config <C>] [--set name=value]... [--json]`** evaluates the `body:` of a `CalculationDef` whose `bodyLanguage` is absent, `budget` or `syscribe`, with its `in` parameters bound from `--set`, else their `value:` defaults, over the same attribute/configuration environment as `check`. It prints the result in SI base units (`--json`: `{calculation, value, unit, dimension, bindings}`). A parameter without a value, or an evaluation error, exits `2` with the message; a name that is not a `CalculationDef`, or an opaque body language (`kerml`, `alf`, `ocl`), exits `1`.
- **`constraints sweep --param name=<lo>..<hi>:<step> [--param …] [--config <C>] [--set …] [--show '<expr>'] [--json]`** evaluates the whole check set once per point of the grid spanned by the `--param` ranges (`3..7:1`, `1000kg..2000kg:250kg`, or a list `1,2,5`), each point binding the named values last. A range is inclusive of `hi`, its step is positive and of the bounds' dimension. The table (or `--json` `points:[{values, summary, show}]`) gives per point the counts of pass/fail/unresolved/error and, with `--show`, the value of an expression at that point. More than 5000 points, or a malformed range, is a usage error (exit `1`); a sweep otherwise exits `0` — it reports, it does not gate.
