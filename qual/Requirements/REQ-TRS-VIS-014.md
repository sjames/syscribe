---
id: REQ-TRS-VIS-014
type: Requirement
name: Every visualisation layer is tested with golden IR, writer snapshots, endpoint integration and a Node-side ELK layout check
status: verified
reqDomain: software
verificationMethod: test
---

The visualisation stack **shall** carry tests at each layer, run by `cargo test --workspace` and
the frontend's `npm test`: the manifest parser (every §8.16.3/8.16.4 form, `E405`/`W416`), the
generators (temp-built fixture models with golden JSON snapshots of the IR under
`crates/syscribe-model/tests/vis_snapshots/`), the writers (PlantUML, Mermaid and SVG snapshots),
the embedded layout engine (clean layouts, Node-vs-QuickJS determinism, the vendored bundle pin),
the server (`GET /api/diagrams/model`, `PATCH`/`DELETE` layout, `PUT` svg) and the client (ELK
layout check, connect rules, server sizes).

**Source:** `REQ-TRS-VIS-014` (product model).

**Acceptance criteria:** the named test files exist and contain tests; the snapshot directories
are populated and the Node-produced ELK output is committed; `npm test` runs every Node test.
Verified structurally by `TC-TRS-VIS-014` and functionally by every `TC-TRS-VIS-*` case that
hosts those files.
