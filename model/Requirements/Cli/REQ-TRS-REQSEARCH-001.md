---
type: Requirement
id: REQ-TRS-REQSEARCH-001
name: "The Requirements Explorer can search elements and filter the drawn graph"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server` shall let the user find a root by search and narrow the drawn graph by node attributes (GH #269, phase 3).

## Behavior

- `GET /api/req-graph/search?q=<text>[&limit=N][&config=C]` returns `{"results":[{id,qname,type,name,status}],"truncated":bool}`: elements whose id, name or qualified name contains `q` (case-insensitive). Results are ranked exact id, id prefix, id substring, name substring, qualified-name substring, then by id; `limit` defaults to 20 and is capped at 100. An empty or missing `q` is 400, a non-numeric `limit` is 400, an unknown configuration is 400, and elements inactive in `config` are not returned.
- The explorer page has a search box that lists matches and focuses the graph on the chosen one (pushing a history entry). Responses to superseded searches are dropped.
- The page filters the drawn graph on the client by element type, verification state and ASIL: non-matching nodes are hidden together with their edges, the root always stays, and a count of hidden nodes is shown. Filtering never re-queries the server. The filter is a pure function of the API response and the filter values.
- The graph API also draws the safety and security relations as edges, filterable by kind: `hazardRef`, `hazardousEvents`, `threatRef`, `threatScenarios`, `mitigatedBy`, `derivedFromCybersecurityGoal`, `relatedSafetyGoal`, `confirms`, `implementedBy`.
