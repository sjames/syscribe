---
type: Requirement
id: REQ-TRS-STYPE-001
name: "Type and unit errors in executable expressions are found at validation time, not on the paths a run happens to take"
status: draft
reqDomain: software
reqClass: system
tags:
  - execution
  - expressions
---

The expressions of an executable state machine shall be type- and dimension-checked statically (GH #272 backlog): `validate`, `compile` and `simulate` report `E935` for an expression that the evaluator would reject at run time whatever the values, without running anything. Rationale and rules: `docs/design/static-typing.md`.

## Behavior

- **Types.** A type is Boolean, String, a number with a known dimension, a number of unknown dimension, or unknown. Variables take the type of their declared initial value; names that are not variables (event payload fields, attribute paths, enumeration literals) are unknown, and unknown is compatible with everything, so an expression is rejected only when the error is certain whenever evaluation reaches the offending operator.
- **Rules**, mirroring the evaluator (`and`, `or` and `if` evaluate their right/else operands lazily, so a type error there is a latent error, still reported, since it fails the evaluation whenever that operand is reached): `not`, `and`, `or` and `if` conditions need Boolean operands; unary minus and arithmetic need numbers; `+`, `-`, `%` need equal dimensions, `*` and `/` combine them; `<`, `<=`, `>`, `>=` need two numbers of equal dimension, `==` and `!=` two numbers of equal dimension, two Booleans or two strings; `min`/`max` need equal dimensions; `sqrt` needs even exponents; `pow` needs a dimensionless exponent (an integer literal when it is a literal).
- **Where.** Transition guards and `when` conditions must be Boolean; the right-hand side of an assignment must have the dimension and kind of the variable assigned; `send` payload expressions must type-check; entry, exit and initial actions are checked like transition actions. Sub-expressions of any of these are checked recursively.
- **Code.** `E935`, an error, one finding per offending expression, naming the transition or action and the rule broken. It is reported together with any other lowering diagnostic of the machine, not after it.
- **No new false positives.** Only executable machines are checked (those with `variables:`, `after`, `when`, `internal`, `assign`, `send`, …); an expression is rejected only if the evaluator fails on it for some values of its variables at the offending operator (a latent error in a lazily evaluated operand counts; a branch that constant folding would prove dead, such as `false and (1 s + 1)`, is still reported); a differential test (`typecheck_diff`) checks the rules against the evaluator over every operator and operand type.
- **Not in v1:** dimension inference through payload fields, constants folded across variables. Action flows are REQ-TRS-STYPE-002.
