---
type: Requirement
id: REQ-TRS-PIQ-002
name: "questions lists the open questions across the model"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

`syscribe questions` shall list the questions of every `PlanningItem`, and `list`/`show` shall surface them (GH #436).

## Behavior

- `questions [--open] [--status S] [--item <PI>] [--json]` prints one row per question with its owning item (`PI-X#Q1`), status, text, the recommendation and the answer. `--open` keeps `open` ones; `--item` restricts to one item (id or qualified name); an unknown item exits 1.
- `list PlanningItem --open-questions` keeps only items with at least one `open` question, and the `--json` form carries the count.
- `show <PI>` prints its questions. Items without any print nothing extra.
- `validate --hints` raises `I913` for an item whose body has a *Questions* heading while `questions:` is empty.
