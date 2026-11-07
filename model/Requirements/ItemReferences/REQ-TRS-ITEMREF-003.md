---
type: Requirement
id: REQ-TRS-ITEMREF-003
name: "Citations of open or answered questions are checked against the citing element"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
  - references
---

`validate` shall flag question citations that no longer match the question's state (GH #453).

1. {#open-question} **Open question from a committed element.** An element whose status is past draft cites a question that is still `open`: `W093`. Uncommitted statuses are none, `draft`, `proposed`, `todo`, `in_progress`, `blocked`, `open` and the retiring statuses (`deprecated`, `superseded`, `retired`, `obsolete`, `withdrawn`, `rejected`). A PlanningItem citing its own question is exempt (`W317` covers a `done` item).
2. {#stale-wording} **Stale confirmation wording.** An `answered` question is cited with a "to confirm"-style phrase in the same sentence: `W094`. Default phrases are "to confirm", "to be confirmed", "TBC", "pending confirmation" and "awaiting", matched case-insensitively on word boundaries.
3. {#markers-config} **Configuration.** `[references] open_markers = [...]` in `.syscribe.toml` replaces the default phrases; an empty list turns `W094` off. A malformed `[references]` entry is reported as `W094` against `.syscribe.toml` and ignored.
