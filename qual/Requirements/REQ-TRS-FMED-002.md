---
id: REQ-TRS-FMED-002
type: Requirement
name: Dead, core, false-optional and void are marked on the feature diagram, each with the reason, and update live
status: verified
reqDomain: software
verificationMethod: test
---

`GET /api/feature-model/analysis` **shall** return, from the SAT engine, whether the model is void and each feature's state (`dead`, `core`, `falseOptional` or `normal`) with the constraints responsible as human labels, and for a void model the conflict and the corrections. The `/features` page **shall** mark each state on the diagram, show the reason on selection, show a banner for a void or skipped analysis, and refresh when the model reloads. A mandatory feature with the default group kind **shall not** be reported false-optional.

**Source:** `REQ-TRS-FMED-002` (product model).

**Acceptance criteria:** (a) a dead feature is reported dead with its excluding constraint as the reason, a feature required by a core feature is core and false-optional, and counts add up; (b) a void model reports its conflict and diagnoses; (c) a model without features reports none and is not an error; (d) the analysis is overlaid on the diagram by qualified name and cleared without a report; (e) the banner and summary text follow the report; (f) a mandatory feature with the default group kind is core, not false-optional.
