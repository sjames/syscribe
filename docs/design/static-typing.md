# Static typing of executable expressions — design

GH #272 backlog, requirement `REQ-TRS-STYPE-001`, test case `TC-TRS-STYPE-001`.

## Problem

The expression language (`expr`) is dynamically typed: a guard such as `height < 5` with `height` in metres fails only when a run reaches it. In a model that is simulated through a few scenarios, such errors survive until the path is exercised, which is exactly what weak scenario sets do not do.

## Approach

`typecheck::infer` applies the evaluator's rules to *types* instead of values. A type is `Bool`, `Str`, `Num(dimension)`, `Num(unknown dimension)` or `Unknown`.

* Variables take the type of their declared initial value (the IR keeps it); every other name (payload fields, attribute paths, enumeration literals) is `Unknown`. `Unknown` is compatible with everything, so an error is reported only when it is certain.
* Rules are those of `expr::compare`, `arith` and `call`: comparisons and `+ - %` need equal dimensions, `* /` combine them, `and`/`or`/`not`/`if` need booleans, `min`/`max` equal dimensions, `sqrt` even exponents, `pow` a dimensionless (literal: integer) exponent.
* `and`, `or` and `if` evaluate lazily, so a type error in their right/else operand fails an evaluation only when reached. It is still reported: it is a latent error, and finding such paths is the point.

## Where

`ir::lower` runs `typecheck::machine_diags` on the finished machine (guards, `when` conditions, assignments against the variable's type, `send` payloads, entry/exit/initial actions), so `compile` and `simulate` refuse what `validate` reports. The validator lowers executable `StateDef`s and shows only `E935` (the other lowering diagnostics are the schema checks, reported once elsewhere).

## Why it can be trusted

`crates/syscribe-model/tests/typecheck_diff.rs` compares the checker with the evaluator for every operator, function and operand type over a spread of values, under both truth values of a boolean operand: a static error requires a runtime type or dimension error for at least one of them, and a static success requires none for both, with matching result types and dimensions.

## Not in the first version

Action flows (`ActionDef`): their expressions have the same rules but parameters are untyped, so the benefit is smaller; dimension inference through payload fields; constants propagated across variables.
