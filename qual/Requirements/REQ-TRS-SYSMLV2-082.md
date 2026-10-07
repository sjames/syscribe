---
id: REQ-TRS-SYSMLV2-082
type: Requirement
name: "then fork, join, decide, accept, send and if produce nodes and succession edges"
status: verified
reqDomain: software
verificationMethod: test
---

In an action body, `then fork f;`, `then join j;`, `then decide d;`, `then accept ...;`, `then send ...;` and `then if c { ... }` shall be ingested as the corresponding `controlNodes:`/`subActions:` entry plus a `successionConnections:` edge from the preceding node, instead of being dropped. A target without a name (an anonymous `then fork;`) still adds no edge.

**Source:** `REQ-TRS-SYSMLV2-082` (product model), `ADR-SYS-SYSMLV2-001`.
