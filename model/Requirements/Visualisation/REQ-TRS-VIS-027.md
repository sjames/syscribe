---
type: Requirement
id: REQ-TRS-VIS-027
name: "A live planning dashboard shows every PlanningItem by status, who is working on it and which agents are working now"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - planning
  - dashboard
  - visualisation
---

The server **shall** serve a separate page, `GET /planning`, and a refreshable fragment,
`GET /ui/planning/board`, that present the model's `PlanningItem`s as a dashboard:

- **Summary:** a count per status, and the total.
- **Working now:** every item that is `in_progress` or has a non-empty `claimedBy:`, grouped by who
  is on it. A claimant (an agent) is shown with a live indicator and the age of its `claimedAt:`;
  an `assignedTo:` person is shown by the display name from the `[users]` roster when declared.
  An item with no assignee and no claimant is grouped as unassigned.
- **Board:** one column per status (`todo`, `in_progress`, `blocked`, `done`, then any other
  status found), each item a card with its id, name, item type, parent, assignee, claimant and the
  items it is `blockedBy:`. A card opens the item's detail dialog.
- **Filters:** `?who=<assignee-or-claimant>` and `?done=0|1` (done items hidden by default, their
  count still shown).

The page **shall** refresh without a manual reload: it re-fetches the board on every model-reload
event of the live-reload WebSocket (so a `claim`, `release`, status edit or an agent's write
appears at once) and on a timer so claim ages stay current. A refresh keeps the active filters.
Every user-controlled string is HTML-escaped. A model with no `PlanningItem` shows an empty-state
message, not an error.

## Rationale

`claimedBy:`/`claimedAt:` already let concurrent agents announce what they are working on.
Showing that live makes the model's own work-tracking visible to the humans and agents sharing it,
and gives a place to watch an agent working on content as it happens.

## Scope

Read-only. Changing status or claims stays in the CLI, MCP and the detail dialog.
