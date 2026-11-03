---
type: Requirement
id: REQ-TRS-PIQ-003
name: "set records, answers and defers questions without hand-editing YAML"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PLANITEM-000]
breakdownAdr: Decisions::PlanningItemADR
tags:
  - planning
---

`syscribe set <PI>` shall add, answer and defer inline questions, keeping the file's comments and order (GH #436).

## Behavior

- `question.add text=<t> [recommended=<r>]` appends an `open` entry with the next free `Q<n>`; `question.answer <Qn> text=<a> [by=<actor>]` sets `status: answered`, `answer`, `answeredAt` and, when given, `answeredBy`; `question.defer <Qn>` sets `status: deferred`.
- Only a `PlanningItem` takes these operations; an unknown `Qn` or an empty text is refused with exit 1 and nothing written. Answering an already `answered` question replaces its answer and refreshes `answeredAt`. `--dry-run` previews the diff.
- Everything outside the edited entry is byte-identical.
