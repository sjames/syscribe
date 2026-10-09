---
type: Diagram
name: TraceabilityEngine
diagramKind: Traceability
subject: Safety::HARA::SG-ENG-001
---

Hazard-to-test traceability for the safety goal `SG-ENG-001` **derived from the model**: the hazardous
event the goal answers, the requirements derived from it and their decomposition, the test cases that
verify them, and the goal's fault tree and argument. Each node is coloured by what is below it (a
failing or missing test is red, an unverified branch amber, a fully passing one green) and a gap is a
badge naming the finding the validator raises — `W002`/`W003` (no test), `W305` (no integration
test), `W300` (nothing satisfies a leaf). Test verdicts read `unknown` here, since a derived diagram
has no results sidecar; `syscribe trace --format dot` reads the ingested ones.
