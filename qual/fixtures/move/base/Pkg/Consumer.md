---
type: PartDef
name: Consumer
features:
  - name: w
    type: Part
    typedBy: Pkg::Sub::Widget
  - name: selfPort
    type: Port
connections:
  - from: Pkg::Consumer::selfPort
    to: Pkg::Sub::Widget::port
---
Consumes a Widget and connects to its port.
