---
type: Requirement
id: REQ-TRS-PIQ-001
name: "A PlanningItem carries inline questions that validate checks"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

A `PlanningItem` shall be able to carry a `questions:` list — the decisions awaiting an answer — that `validate` reads and checks (GH #436, `ADR-SYS-PLANITEM-001` addendum).

## Behavior

- Each entry is `{id, text, recommended?, status, answer?, answeredBy?, answeredAt?}`; `id` is local to the item (`Q<n>`), `status` is `open | answered | deferred`. A question is private to its item and is not a cross-reference target.
- `E724`: `questions:` is not a list, or an entry is not a mapping, lacks `id` or `text`, or its `id` is not `Q<n>`. `E725`: two entries share an `id`. `E726`: `status` is outside the vocabulary (or absent). `E727`: an `answered` entry has no non-empty `answer`.
- `W318`: a `questions:` field on an element that is not a `PlanningItem` is ignored there and warned about, so a misplaced decision is not silently lost.
- `W317`: an item at `status: done` has an `open` question. `deferred` and `answered` raise nothing. An open question never makes an item `blocked`.
- `answeredBy` is an opaque actor id and `answeredAt` a free timestamp string; neither is validated.
- Items without `questions:` behave exactly as before.
