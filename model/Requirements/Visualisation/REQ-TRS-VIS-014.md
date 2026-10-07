---
type: Requirement
id: REQ-TRS-VIS-014
name: "Every visualisation layer is tested: golden IR, writer snapshots, endpoint integration and a Node-side ELK layout check"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-000]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - testing
---

The visualisation stack shall carry tests at each layer, run by `cargo test --workspace` and
the frontend's `npm test`:

- **Manifest parser** — every §8.16.3/8.16.4 form, and one test per `E405`/`W416` trigger.
- **Generators** — temp-built fixture models in the tests, with golden JSON snapshots of the
  IR under `crates/syscribe-model/tests/vis_snapshots/`, one assertion per rule of
  `REQ-TRS-VIS-004`/`-005`, including `include:`/`exclude:` and the deterministic shape ids.
- **Writers** — snapshot tests for PlantUML, Mermaid and SVG on the same fixture IRs; the
  PlantUML before/after snapshot for the demo model's companion diagrams.
- **Server** — Axum integration tests on `GET /api/diagrams/model` (nesting, ports,
  `layoutOptions`, `pinned`) and on `PATCH`/`DELETE` layout.
- **Client** — `npm run typecheck` remains the build gate; a Node script feeds a fixture
  `SGraph` through `elkjs` with the production layout options and asserts that no two sibling
  nodes overlap and every port lies on its parent's border.

Qualification mirrors (`qual/Requirements`, `qual/TestCases` with real `testFunctions`) shall
accompany each phase, as for every other feature.

## Rationale

The 2026-10-07 review counted a handful of parse-helper unit tests across ~5 k lines of
rendering code. A rebuild without tests at each boundary would reproduce the same drift the
rebuild exists to end.

## Scope

- The Node test is a dev-time step like the esbuild build; it adds no runtime dependency.
- Screenshot-based browser testing is not required.
