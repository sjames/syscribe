---
type: Requirement
id: REQ-SIL-SEC-002
title: "Field bus commands shall include EN 50159 safety codes preventing replay and insertion"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-SIL-SYS-000
breakdownAdr: ADR-SIL-COMM-001
derivedFromSecurityGoal: CSG-SIL-002
silLevel: 4
tags:
  - security
  - field-bus
  - EN50159
  - replay-protection
---

All commands sent from the vital processor output stage to object controllers (point machines, signal heads, track circuit interfaces) over the field bus **shall** include EN 50159 Category 2 safety codes as specified in this requirement. The field bus **shall** reject any message that does not carry a valid Category 2 safety code frame.

The Category 2 safety code frame **shall** include, at minimum:

- A 32-bit CRC computed over the complete message payload (including all header fields) using the CRC-32/AUTOSAR polynomial, providing a residual error probability of ≤ 10⁻⁹ per message.
- A 32-bit monotonically-increasing sequence counter, unique per ordered source–destination pair. The receiving object controller **shall** reject any message whose sequence counter is not strictly greater than the last accepted sequence counter from that source.
- A 32-bit timestamp in milliseconds since the last PTP synchronisation epoch, derived from an IEEE 1588 Precision Time Protocol grandmaster clock. The receiving object controller **shall** reject any message whose timestamp differs from the receiver's local clock by more than ±2000 ms.
- A 16-bit source address uniquely identifying the transmitting vital processor channel (Channel A or Channel B) and the transmitting logical node index.
- A 16-bit destination address uniquely identifying the intended recipient object controller.

Any field bus device that presents an unexpected source address in the source address field — that is, a source address not corresponding to a known vital processor channel — **shall** have its messages rejected by the field bus switch's port filtering, and **shall** trigger an alarm to the maintainer workstation.

The field bus **shall** be physically restricted to the interlocking equipment room (IER) and the designated lineside cable routes. All field bus cable outside the IER **shall** be installed in tamper-evident conduit. Conduit integrity checks **shall** be included in the scheduled maintenance plan.

The implementation shall satisfy security controls SC-SIL-002 and SC-SIL-003.

## Rationale

The EN 50159 Category 2 safety codes serve a **dual purpose**: they provide safety integrity against EMC-induced corruption of field bus messages, and they simultaneously provide the cybersecurity replay and insertion protection required by the threat model.

The sequence counter prevents replay attacks: an attacker who captures a valid point-movement command over the lineside cable cannot replay it later, because the receiving object controller will reject any message with a sequence counter equal to or less than the most recently accepted counter. The replay window is bounded to a single 20 ms execution cycle — the time between successive valid messages. Within this window, a replayed message carries the same command as the legitimate message, so it causes no additional harm.

The timestamp provides a secondary defence against attacks involving delayed delivery of captured messages (e.g., recording a valid command during maintenance and replaying it during operational service). Even if the sequence counter has advanced past the captured value, the timestamp check limits the replay window to ±2 seconds regardless.

The source address field prevents insertion attacks: a device connected to the field bus but not identifiable as a vital processor channel will be rejected at both the switch port filter and the object controller. This defends against the scenario where an attacker connects a rogue device to a lineside junction box.

This requirement is the software-level expression of the architectural decision in ADR-SIL-COMM-001.

## Notes

- Verification method `test` shall be demonstrated on the Hardware-in-the-Loop (HIL) test bench by injecting replayed messages (same sequence counter, different sequence counter, expired timestamp) and verifying rejection at the object controller. Test cases are defined in the Verification package.
- The ±2000 ms clock synchronisation tolerance is achievable with IEEE 1588 PTP on industrial Ethernet; typical achieved synchronisation accuracy is ±100 µs. The ±2000 ms value is a conservative tolerance to accommodate clock drift during a PTP grandmaster failover.
- Physical tamper-evident conduit is a defence-in-depth measure; it does not prevent a sophisticated attacker with time and tools but substantially increases both detection probability and attack difficulty, reducing the feasibility rating of TS-SIL-002 from `medium` to `low`.
