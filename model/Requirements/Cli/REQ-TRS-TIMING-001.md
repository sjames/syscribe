---
type: Requirement
id: REQ-TRS-TIMING-001
name: "timing measures load and validation so the need for incremental revalidation is a number"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - performance
---

`syscribe timing [--json]` shall load the model and validate it once and report each phase — load (walk, parse, synthesis passes), validate, and the validation of every Configuration's variant — with the file, element, Configuration and finding counts, the peak memory and whether the build is debug or release (GH #429).

- A guard in the test suite asserts generous ceilings on the repository's own models and that doubling a synthetic model does not more than triple the time, so a regression to super-linear behaviour fails; an ignored test prints the scaling table.
- The measurements, the target and the decision not to build incremental revalidation yet — with the trigger for building it and the equivalence test it would need — are recorded in `docs/design/incremental-validation.md`.
