---
type: Requirement
id: REQ-SIL-SW-004
name: Safety communication shall implement EN 50159 Category 2 safety codes
status: approved
reqDomain: software
silLevel: 4
verificationMethod: test
derivedFrom:
  - REQ-SIL-SAFE-000
breakdownAdr: ADR-SIL-COMM-001
---

All safety-relevant communication between the vital processor channels and between the vital processors and object controllers **shall** implement EN 50159 Category 2 safety codes. Each message **shall** include: a 32-bit CRC (polynomial per EN 50159 Annex C), a 32-bit monotonic sequence number, a 32-bit timestamp (1ms resolution), and a source/destination address pair. The receiver **shall** detect and reject: corrupted messages (CRC mismatch), repeated messages (sequence number not incrementing), delayed messages (timestamp outside ±2× defined safety time window), and messages from unexpected sources (address mismatch). Any detected safety communication error **shall** be treated identically to a channel comparison failure — both channels revert to safe state. The safety time window **shall** be configured to ≤ 150ms for the cross-comparison bus and ≤ 100ms for the field bus.
