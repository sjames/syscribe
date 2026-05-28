---
type: Requirement
id: REQ-SIL-SW-001
title: CBI vital processing shall use 2oo2D voting architecture with diverse processor channels
status: approved
reqDomain: hardware
silLevel: 4
verificationMethod: inspection
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-SYS-001
---

The vital processing hardware **shall** implement a 2oo2D (two-out-of-two with diagnostics) architecture. Two independent processing channels (Channel A and Channel B) **shall** independently execute the interlocking vital logic on every scan cycle and compare their output state vectors via the cross-comparison bus before asserting any output. Any discrepancy between the two channels' output vectors **shall** cause both channels to assert the safe state simultaneously within one scan cycle (≤50ms). The comparison mechanism **shall** itself be designed to SIL 4 integrity. Channel hardware **shall** be physically diverse (different board designs, different supply regulators) to defend against common-cause hardware failures. Reference: EN 50129 §5.4 (diverse redundancy).
