---
type: Diagram
name: SysIBD
diagramKind: IBD
subject: Sys
shapes:
  s-sys:
    ref: Sys
    kind: boundary
  s-engine:
    ref: Sys::Engine
    kind: block
    parent: s-sys
  s-pout:
    ref: Sys::PowerOut
    kind: port
    parent: s-engine
  s-motor:
    ref: Sys::Motor
    kind: block
    parent: s-sys
  s-ghost:
    ref: Nowhere::Missing
    kind: block
    parent: s-sys
edges:
  e-flow:
    source: s-pout
    target: s-motor
    kind: flow
layout:
  s-motor: {x: 220, y: 40, w: 150, h: 60}
  e-flow: {points: [[100, 50]]}
---

IBD fixture: a boundary holding two blocks, one of which carries a port; the
motor is pinned with a size, nothing else is pinned.
