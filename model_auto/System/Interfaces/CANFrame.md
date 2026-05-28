---
type: ItemDef
name: CAN Frame
features:
  - name: arbitrationId
    type: ScalarValues::Integer
  - name: dlc
    type: ScalarValues::Integer
  - name: payload
    type: ScalarValues::String
---

CAN data frame conforming to ISO 11898-1. Carries up to 8 bytes of payload.

Safety-critical PDUs (torque command, brake request) carry a 24-bit CMAC-AES-128
truncated message authentication code in the last 3 bytes of payload, per
`Security::TARA-ENG-001` control `SC-ENG-001`.
