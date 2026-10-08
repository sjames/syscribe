---
id: REQ-TRS-VIS-027
type: Requirement
name: A live planning dashboard shows every PlanningItem by status, who is working on it and which agents are working now
status: verified
reqDomain: software
verificationMethod: test
---

The server **shall** serve `GET /planning` and a refreshable `GET /ui/planning/board` presenting
every `PlanningItem` with a count per status, a "working now" grouping of `in_progress` or claimed
items by agent claimant (with `claimedAt` age) or assigned person (roster display name), a column
per status with cards (id, name, type, parent, assignee, claimant, `blockedBy`) that open the
detail dialog, and `who`/`done` filters (done hidden by default, still counted). The page
**shall** re-fetch the board on each model-reload event and on a timer, keeping filters; every
user-controlled string **shall** be escaped; a model without items **shall** show an empty state.

**Source:** `REQ-TRS-VIS-027` (product model).

**Acceptance criteria:** (a) the page is served with the board host, script, live-reload client and
header link; (b) the fragment counts every status and groups agent, person and unassigned work;
(c) blocked cards name what they wait on and done cards are hidden but counted, shown on request;
(d) `who` keeps only that person's or agent's items; (e) text such as `<b>` is escaped; (f) an
empty model says so; (g) a changed claim shows on the next fetch; (h) the client helpers build
the filtered URL, format ages and keep the current filter in the people list.
