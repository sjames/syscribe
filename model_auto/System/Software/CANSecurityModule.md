---
type: PartDef
name: CAN Security Module
domain: software
satisfies:
  - REQ-ENG-SEC-001
features:
  - name: macAlgorithm
    type: ScalarValues::String
  - name: keyLength
    type: ScalarValues::Integer
    unit: bits
---

Software module implementing message authentication for safety-critical CAN
frames on the powertrain network.

## Authentication scheme

Uses AUTOSAR SecOC (Secure Onboard Communication) with CMAC-AES-128.
Each outbound safety-critical PDU carries a 24-bit truncated MAC and a
4-bit freshness counter. Inbound frames with invalid MAC are rejected and
a security DTC is set.

Messages protected: torque requests, brake-force requests, transmission
shift commands.
