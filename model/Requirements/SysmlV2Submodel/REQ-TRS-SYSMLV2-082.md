---
type: Requirement
id: REQ-TRS-SYSMLV2-082
name: "then fork, join, decide, accept, send and if produce nodes and succession edges"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

In an action body, `then fork f;`, `then join j;`, `then decide d;`, `then accept ...;`, `then send ...;` and `then if c { ... }` shall be ingested as the corresponding `controlNodes:`/`subActions:` entry plus a `successionConnections:` edge from the preceding node, instead of being dropped. A target without a name (an anonymous `then fork;`) still adds no edge.
