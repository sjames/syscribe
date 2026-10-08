---
id: REQ-TRS-FMED-006
type: Requirement
name: Two configurations are compared and features are shown against configurations in a matrix
status: verified
reqDomain: software
verificationMethod: test
---

The page **shall** show a matrix of the features the diagram shows against every stored configuration (chosen on, chosen off, not mentioned), following the diagram's collapse and filtered by name, id or qualified name keeping ancestors; and the comparison of two chosen configurations, highlighting the features on which they differ (a feature not mentioned counting as off) and listing what only each selects.

**Source:** `REQ-TRS-FMED-006` (product model).

**Acceptance criteria:** (a) rows follow tree order and carry depth and each configuration's choice; (b) a collapsed subtree is absent and a query keeps matches with their ancestors; (c) the two compared configurations' differences are marked and listed with counts of features in both and neither; (d) a cell reads as tick, cross or dot.
