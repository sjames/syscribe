---
id: REQ-TRS-FMED-007
type: Requirement
name: Models of thousands of features stay responsive
status: verified
reqDomain: software
verificationMethod: test
---

A feature model of 2,000 features and 300 constraints **shall** be analysed, configured and drawn within an interactive time: the analysis limit **shall** admit it, each satisfying model **shall** witness every feature at once, an unsatisfiable core **shall** be found on one incremental solver, a model of more than 60 features **shall** open collapsed to two levels so only visible features are laid out, and a diagram too wide to read **shall** be shown from its first root at a readable zoom.

**Source:** `REQ-TRS-FMED-007` (product model).

**Acceptance criteria:** (a) the analysis limit is at least 2,000 features; (b) a 2,000-feature model is analysed, a selection propagated and a conflict explained inside generous ceilings in a debug build; (c) the roots are found, one more level can be opened, and a diagram that would fit below 30% zoom is shown from its root.
