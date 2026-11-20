---
type: Requirement
id: REQ-TRS-PLANCOV-001
name: "A TestPlan declares the coverage it requires: conditions, minimum units and how they combine"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - testplan
---

A `TestPlan` shall accept `conditions:` (named axes, each a list of named points with an optional value), `minUnits:` (an integer of at least 1), `mode:` (`each`, the default, or `product`) and `points:` (an explicit list of required coordinate sets, each a map of axis to point) as the coverage it requires (ADR-SYS-COVERAGE-001, GH #500, GH #501). `each` requires every point of every axis on its own; `product` requires every combination of one point per axis; `points:` replaces `mode:` as the definition of what is required. A malformed declaration shall be `E607`: a `conditions:` that is not a map of axis to a non-empty list of uniquely named points, a `minUnits:` below 1 or not an integer, a `mode:` other than `each` or `product`, `points:` that is not a list of maps, `points:` together with `mode:`, or `mode:`/`points:` with no axis. A `points:` entry that names an axis or a point `conditions:` does not declare shall be `E608`. On any other element type the four keys shall stay unrecognised (`W047`). Judging a plan against the declaration is a later slice of the same decision.
