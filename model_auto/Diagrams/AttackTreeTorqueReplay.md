---
type: Diagram
name: AttackTreeTorqueReplay
diagramKind: AttackTree
subject: Security::Attacks::AT-ENG-001
---

Attack tree for the threat `TS-ENG-001` **derived from the model**: each gate and step is coloured by
its rolled-up attack feasibility (weakest link: an AND path is as hard as its hardest step, an OR
as easy as its easiest alternative), the easiest path from the root is drawn heavier, and the root
says whether its computed feasibility matches the one the TARA declares (otherwise `W035`).
