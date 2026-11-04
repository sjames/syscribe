---
type: Requirement
id: REQ-TRS-CHILDREN-001
name: "children returns a tree node's direct children with counters, one page per call"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe children [<node>] [--view containment|configurations] [--offset N] [--limit N] [--json]`, the MCP tool `children` and the LSP request `syscribe/children` shall return the direct children of one tree node, so an explorer needs one call per expanded node (GH #425).

- A child carries id, type, status, file, whether it is a package and has children, and counters: children, derived children, verifying tests, errors and warnings (for a package, those of its subtree). A directory with no `_index.md` is a virtual package.
- The `configurations` view is a tree following `derivedFrom:` and `subConfigurations:`.
- Results are paginated (`offset`, `limit`, `nextOffset`); the pages cover the children exactly once. The payload has the published `children` schema, and nothing is written.
