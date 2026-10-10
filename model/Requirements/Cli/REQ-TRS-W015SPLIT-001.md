---
type: Requirement
id: REQ-TRS-W015SPLIT-001
name: "W015 says whether a parent is uncovered directly only or also below"
status: draft
reqDomain: software
reqClass: system
tags:
  - validation
  - coverage
---

`W015` shall distinguish "no direct test" from "no verified leaves below" for a parent requirement (GH #252, message split).

## Behavior

- For a leaf requirement the message is unchanged.
- For a parent requirement (one with derived children) that no TestCase covers in some configuration, the message adds, for those configurations: `its leaves below are all verified there — covered through its children only, it still needs a direct test` when every leaf descendant active there has a covering TestCase, else `and not every leaf below it is verified there`.
- Which findings are raised does not change; only the message text.
