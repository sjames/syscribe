---
type: Requirement
id: REQ-TRS-VIEWTAB-003
name: "validate checks a table view's declaration and the fields it names"
status: approved
reqDomain: software
reqClass: system
tags:
  - validation
  - view
---

`validate` shall check the table declaration of a `View`/`ViewDef` (`REQ-TRS-VIEWTAB-001`, GH #531).

1. {#e507} A malformed declaration is error `E507`, naming what is wrong: no `columns:`, a column that is neither a field name nor a well-formed map, an unknown column key, a `sort:`, `rows:`, `where:`, `caption:`, `label:`, `nest:` or `marks:` of the wrong shape, a condition clause that is not a comparison, or `nest:` with `rows:`. `view render` refuses such a view.
2. {#w504} A column, a `sort:` key, `nest:`, or a field of a `where:` or `marks:` condition that no row can have is warning `W504`; so is a `rows:` list that none of the exposed elements carries. A `custom.<key>` field, and a field of a list none of whose entries exist yet (other than `questions` and `parameters`), are not reported.
3. {#keys} The declaration keys are schema fields on a `View`/`ViewDef` (no `W047`) and stay unrecognised on any other type.
