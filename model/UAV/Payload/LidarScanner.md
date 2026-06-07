---
type: PartDef
name: LidarScanner
supertype: Parts::Part
domain: hardware
features:
  - name: pointRatePtsPerSec
    typedBy: ScalarValues::Integer
    isReadonly: true
    isConstant: true
  - name: powerIn
    type: Port
    typedBy: Interfaces::PowerPortReceiverDef
    direction: in
---

Rotating LiDAR scanner producing a 3-D point cloud. Point rate is a fixed
hardware parameter. Used by the mapping payload variant.
