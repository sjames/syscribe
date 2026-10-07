---
type: Package
name: Visualisation
---

Requirements for the visualisation rebuild: one Diagram intermediate representation in
`syscribe-model` built either from a hand-listed manifest or derived from the model, laid out
by ELK inside the sprotty browser client, edited through the shared guarded-write engine, and
exported to PlantUML, Mermaid and static SVG from the same representation. Block definition and
internal block diagrams come first.

All requirements derive from `REQ-TRS-VIS-000` and are governed by `ADR-SYS-VIS-001`
(`Decisions::VisualisationADR`); the design is `docs/design/visualisation.md`. The legacy
rendering paths are removed outright, with no backwards-compatibility obligation.

The member requirements are listed by `syscribe show Requirements::Visualisation` (generated
from this directory — not maintained here).
