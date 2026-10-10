---
type: Requirement
id: REQ-TRS-BDDCFG-001
name: "diagram export renders a derived diagram as a configuration and derived blocks show integrity and ownership"
status: draft
reqDomain: software
reqClass: system
tags:
  - vis
---

`diagram export` shall accept `--config <C>` and a derived BDD shall show the integrity allocation of its blocks (GH #244, v1; the grid wrap of wide rows follows).

## Behavior

- `diagram export <qname> --config <C>` projects the model onto Configuration `<C>` (as `show`/`trace --config` do) before deriving the diagram, so elements gated off by `appliesWhen` are absent. An unknown or unsatisfiable configuration, or a diagram that is itself inactive in it, exits 1 with a message. Without `--config` nothing changes.
- A derived BDD block whose definition has `asilLevel` carries an `ASIL <level>` banner, and its compartment lists `domain: <value>` and `responsibility: <value>` lines when those fields are set.
