---
type: Requirement
id: REQ-TRS-ATTRREF-004
name: "A literal equal to a previous value of a linked attribute is W096"
status: draft
reqDomain: software
reqClass: system
tags:
  - references
  - attributes
---

`validate` shall report `W096` (a stale copy) when a non-draft element's body contains a numeric
literal equal to a previous value of an attribute of an element it links to or references, and
different from its current value (GH #452).

1. {#history} **History.** Previous values are read from the committed versions of the file that
   holds the attribute: `git log --follow` and one `git cat-file --batch` per file, at most
   `[agents] audit_limit` commits (default 200), cached per file.
2. {#scope} **Scope.** The links and the noise filter are those of `W095`; `W096` does not need
   `copied_values`. Without git, or outside a repository, nothing is reported.
