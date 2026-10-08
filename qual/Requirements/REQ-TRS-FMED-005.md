---
id: REQ-TRS-FMED-005
type: Requirement
name: For a feature, the elements it gates, the configurations that select it and the effect of removing it are shown
status: verified
reqDomain: software
verificationMethod: test
---

`GET /api/feature-model/impact?feature=` **shall** return, for a feature named by qualified name or `FEAT-*` id, the elements whose `appliesWhen:` names it grouped by type with their identifiers, the elements conditioned only through a package that names it counted per package, the configurations that select and deselect it, the features that require or exclude it and that it requires or excludes, and how many features lie below it. The page **shall** show them with links into the model and offer a preview of removing the feature that reports what would become dead, forced or void and which configurations would become invalid.

**Source:** `REQ-TRS-FMED-005` (product model).

**Acceptance criteria:** (a) a feature's own `appliesWhen:` consumers are grouped by type and a package that names it is itself a gate whose members are counted separately; (b) selecting and deselecting configurations are listed, by id or qualified name; (c) what requires it and what it requires are named; (d) an unknown feature is reported as not found; (e) the summary text reads naturally for none, direct and inherited gates.
