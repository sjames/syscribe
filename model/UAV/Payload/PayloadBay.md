---
type: PartDef
name: PayloadBay
supertype: Parts::Part
features:
  - name: camera
    typedBy: UAV::Payload::Camera
    multiplicity: "0..1"
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
connections:
  - typedBy: Interfaces::PowerConnectionDef
    ends:
      - end: source
        binds: powerIn
      - end: receiver
        binds: camera.powerIn
---

Payload bay supporting an optional electro-optical camera. Provides power and mechanical mounting.
