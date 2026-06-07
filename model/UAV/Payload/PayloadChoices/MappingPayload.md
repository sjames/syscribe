---
type: Part
name: MappingPayload
typedBy: UAV::Payload::LidarScanner
isVariant: true
domain: hardware
appliesWhen: Features::Payload::Mapping
satisfies:
  - REQ-UAV-MAP-001
---

Mapping payload variant — a LiDAR scanner build. Active only in products that
select the `Mapping` feature; satisfies the point-density requirement.
