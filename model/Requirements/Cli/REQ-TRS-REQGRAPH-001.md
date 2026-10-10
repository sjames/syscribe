---
type: Requirement
id: REQ-TRS-REQGRAPH-001
name: "The server serves a bounded traceability neighbourhood graph around a requirement"
status: draft
reqDomain: software
reqClass: system
tags:
  - server
---

`syscribe-server` shall serve the typed traceability graph around an element as JSON, the data layer of the Requirements Explorer (GH #269, phase 1: API and overview; the explorer page, filters and alternate lenses follow).

## Behavior

- `GET /api/req-graph?root=<qname|id>[&depth=N][&edges=k1,k2][&limit=M][&config=C]` returns `{root, nodes[], edges[], truncated}`: the elements within `depth` hops (default 1, at most 6) of `root`, following relations in either direction. A node carries `id`, `qname`, `type`, `name`, `status`, `reqClass`, `asil`, `verification` and `root`; an edge `from`, `to` and `kind`. `verification` is `verified` (a verifying TestCase that is neither draft nor retired, as the matrix classifier counts it), `planned` (only draft ones), `unverified` or `na` (not a requirement). `root` is an exact stable id or qualified name; the edges also carry `fromQname`/`toQname`.
- Edge kinds: `derivedFrom`, `satisfies`, `verifies`, `allocatedTo`, `refines`, `supersedes`, `derivedFromSafetyGoal`, `breakdownAdr`, `blockedBy`, `covers`, `analyses`, `runsOn`, `achieves`, `evidence`, `appliesWhen` (to the gating feature). `edges=` restricts the kinds followed. At most `limit` nodes (default 200) are returned; `truncated` is true when more were reachable.
- `config=C` projects the model onto that Configuration first (inactive elements and the edges to them are absent). An unknown root is 404; a missing `root`, an unknown edge kind, a malformed number or an unknown configuration is 400.
- `GET /api/req-graph/overview` returns counts of requirements by `reqClass` and by `status`, the number of requirements with no relation at all (`unlinked`), and the verification split.
