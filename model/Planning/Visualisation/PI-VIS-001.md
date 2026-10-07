---
type: PlanningItem
id: PI-VIS-001
name: "Phase 0 — Diagram IR, one manifest parser, E405/W416, nested sprotty model, legacy renderers and CLI diagram toolkit removed"
status: done
itemType: feature
achieves: [REQ-TRS-VIS-001, REQ-TRS-VIS-002, REQ-TRS-VIS-013]
evidence:
  - path: "repo:crates/syscribe-model/src/vis/manifest.rs"
  - path: "repo:crates/syscribe-model/tests/vis_manifest_validation.rs"
  - path: "repo:crates/syscribe-model/tests/vis_plantuml_snapshot.rs"
  - path: "repo:crates/syscribe-server/tests/diagram_model.rs"
  - path: "repo:qual/tests/tc/TC-TRS-VIS-013.sh"
tags:
  - visualisation
---

Cleared the ground on 2026-10-07: `syscribe_model::vis::{ir, manifest, sprotty}`, `E405`/`W416`
catalogued, the diagram-model endpoint emitting nesting with a client that renders it,
`sub_mapping` refusing instead of silently replacing a scalar, the PlantUML writer on the IR
(byte-identical output proven by snapshot), and the deletions listed in `REQ-TRS-VIS-013`.
Design: `docs/design/visualisation.md` §5, §9, §11.
