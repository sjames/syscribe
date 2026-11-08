---
type: Requirement
id: REQ-TRS-ITEMREF-004
name: "refs, impact, check-ref and lint-docs understand item and question references"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
  - references
  - cli
---

The query commands shall accept item and question references (GH #453).

1. {#refs} **refs.** `refs REQ-X#slug`, `refs REQ-X#5` and `refs PI-Y#Q3` list the elements citing that item or question; an anchor and a number naming the same item match each other. `refs REQ-X` adds an "Item and question citations" section.
2. {#impact} **impact.** `impact REQ-X#slug` lists the elements citing that item (`via cited by`) and what is downstream of them; upstream it shows the owning element (`via item of`).
3. {#check-ref} **check-ref.** `check-ref REQ-X#slug` (and the other forms) prints the item's number, anchor and first line, or the question's status, text and answer; a reference that does not resolve exits 1.
4. {#lint-docs} **lint-docs.** External Markdown is checked with the same rules: `W092` fails the run, `W094` is advisory unless named by `--deny`.
