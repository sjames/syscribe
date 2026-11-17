---
type: Requirement
id: REQ-TRS-PCFUNC-001
name: "A parameterConstraints expression that calls a function is evaluated, and one with no truth value is an error"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

A `parameterConstraints:` expression shall be evaluated by the expression language of `syscribe eval`, so that a call to `max`, `min`, `ceil`, `floor`, `abs` or `round`, and `if … then … else …`, is judged exactly as the plain-arithmetic form is: `feature-check`, `validate` (so `bind`) and `residual` report a false one as `E221` (`W025` for `severity: warning`) and a true one as satisfied (GH #489).

An expression that has no truth value although every parameter it reads has a value (an unsupported construct, a type or domain error) shall be `E250`, never undetermined and never a silent pass; an expression with a still-open parameter stays undetermined.
