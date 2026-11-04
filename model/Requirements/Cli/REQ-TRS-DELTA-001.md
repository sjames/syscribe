---
type: Requirement
id: REQ-TRS-DELTA-001
name: "The LSP server tells a front end what a reload changed"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

After a reload, `syscribe lsp` shall send a `syscribe/modelChanged` notification to a client that opted in (`capabilities.experimental.syscribe.modelChanged`), listing the qualified names added, changed and removed and whether the feature model, any Configuration or the set of validation findings changed (GH #415).

- The payload conforms to the published `model-changed` schema and carries a `sequence` that increases by one per notification.
- A burst of reloads is reported once after a quiet period; a burst that ends where it began is not reported; a client that did not opt in receives nothing.
- The server still never writes files; the delta is computed from the in-memory model.
