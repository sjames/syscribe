---
type: Requirement
id: REQ-TRS-REQOVER-001
name: "The Requirements Explorer opens on an overview that lists where to start"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server`'s Requirements Explorer shall, when no root is given, show an overview instead of an empty canvas (GH #269, phase 9).

## Behavior

- `GET /api/req-graph/overview` additionally returns `unverifiedList` and `unlinkedList`: the requirements whose `verification` overlay is `unverified`, and those with no relation of the listed edge kinds (the `unlinked` count's population), each as `{id, qname, name, status}` sorted by id and capped at 25 entries; the totals are the existing `verification.unverified` and `unlinked` counts, so a list shorter than its count was capped. Active `config` applies as for the graph.
- With an empty `focus` the page requests the overview and shows the counts by class, status and verification plus the two lists; each entry is a button that focuses the graph on that requirement. With a non-empty focus the overview is not requested. The overview is replaced by the graph as soon as a root is chosen and refreshed on the live-reload event while no root is set.
- While the overview loads the page shows "Loading…" rather than the previous numbers, and says so when the model has no requirements. It stays visible in the table and matrix views when no root is set.
