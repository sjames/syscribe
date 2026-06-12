---
type: Requirement
id: REQ-UAV-MAP-001
name: "Mapping payload shall capture LiDAR point clouds at >= 100 points/m^2"
status: approved
reqDomain: hardware
verificationMethod: test
appliesWhen: Features::Payload::Mapping
derivedFrom:
  - REQ-UAV-VAR-000
breakdownAdr: Decisions::ProductLineADR
tags:
  - mapping
  - payload
---

When configured with the LiDAR mapping payload, the UAV shall capture
georeferenced point clouds at a ground density of at least 100 points per square
metre at the nominal survey altitude.

## Rationale

Photogrammetry-grade deliverables require a minimum point density. This
requirement is only present in products that select the `Mapping` payload.
