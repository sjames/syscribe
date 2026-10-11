---
type: Requirement
id: REQ-GTI-SYS-071
name: "Software self-test (IEC 60730 Class B / UL 1998 style)"
status: approved
reqDomain: software
verificationMethod: test
derivedFrom:
  - REQ-GTI-STK-003
  - REQ-GTI-STK-007
breakdownAdr: ADR-GTI-REQ-001
---

At start-up and periodically the firmware **shall** test the CPU registers, RAM, flash CRC, clock and ADC reference, and **shall** enter the safe state within 100 ms of a detected failure.
