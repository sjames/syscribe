---
type: Requirement
id: REQ-TRS-TIMELINE-001
name: "timeline shows what each stage of a derivedFrom chain decided, with provenance"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe timeline <Configuration> [--json]`, the MCP tool `timeline` and the LSP request `syscribe/timeline` shall show the chain of Configurations the one asked for derives from (root first), what each decided itself and the stage it is bound at, the effective values with their provenance (source kind and the stage that bound them), and what is still open by the stage it can be bound at (GH #418). The payload has the published `timeline` schema; nothing is written.
