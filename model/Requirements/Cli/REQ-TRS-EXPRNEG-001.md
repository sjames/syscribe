---
type: Requirement
id: REQ-TRS-EXPRNEG-001
name: "A minus sign before an offset-scale temperature literal belongs to the number, and a reading below absolute zero is an error"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - expressions
---

In the expression language, a `-` immediately before a number whose unit is an absolute temperature on an offset scale (`degC`, `degF`) shall be part of the literal: `-20 degC` is the absolute temperature 253.15 K, in `eval`, in a guard, a constraint or a calculation, and in a recorded measurement value or limit (GH #499). A literal below 0 K (`-300 degC`, `-460 degF`) shall be an error, not a negative kelvin. Negating a reading expression (`-(20 degC)`) shall remain a dimension error. Every other negative literal (`-5`, `-5 ms`, `-5 K`, `-5 delta_degC`, `-x`) shall parse to exactly the tree it always did, so the hash of a compiled machine, and the evidence recorded against it, does not change. The parser shall be checked against the conversion formulas, a second route through the arithmetic and a separate evaluator over generated expressions.
