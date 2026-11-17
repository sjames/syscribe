---
type: Requirement
id: REQ-TRS-CALCBUILD-001
name: "build-config exports the result of a CalculationDef through its buildVar"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

A `CalculationDef` with `buildVar:` (and no `in` parameters, or all of them defaulted) shall be evaluated by `build-config --config <C>` against the Configuration's effective parameters and emitted in every format (`env`, `makefile`, `cmake`, `c-header`, `kconfig`, `json`) (GH #492), in its `unit:` when one is declared, as a whole number when it is declared `integral: true` (`E251` for a fractional result). A calculation that cannot be evaluated is `E251` and `build-config` exits non-zero (never a silent omission); two exports of one variable with different values are `E050`.
