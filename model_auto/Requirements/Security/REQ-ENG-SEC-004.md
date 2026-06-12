---
type: Requirement
id: REQ-ENG-SEC-004
name: UDS readMemoryByAddress shall be restricted to authenticated diagnostic sessions
status: approved
reqDomain: software
verificationMethod: inspection
derivedFrom:
  - REQ-ENG-SYS-000
breakdownAdr: ADR-ENG-SYS-001
derivedFromSecurityGoal: CSG-ENG-004
---

The Engine ECU **shall** reject any UDS service 0x23 (readMemoryByAddress)
request unless the requesting diagnostic client holds an active security access
session at level 0x11 (or the paired response level 0x12). A rejection **shall**
return UDS Negative Response Code 0x33 (securityAccessDenied) within 100 ms.

Any UDS 0x23 request that is granted **shall** be recorded in a tamper-evident
ring buffer within the ECU's non-volatile memory. The ring buffer **shall**
retain a minimum of 64 entries, each capturing: UTC timestamp (or engine
runtime hours if RTC is unavailable), requested start address, requested
length, session authentication level, and a CMAC integrity tag over the log
entry. When the buffer is full, the oldest entry **shall** be overwritten.

The audit log **shall** be readable only by OEM diagnostic tools operating at
security access level 0x11. Any 0x23 request targeting the calibration data
memory region (precise address range to be specified in the Software Design
Document) while no authenticated session is active **shall** additionally set
a security DTC within 200 ms of the attempt, independent of the NRC already
returned.

A non-authenticated 0x23 attempt to the calibration memory region **shall**
be counted as a security event; after five such events within a single ignition
cycle, the ECU **shall** suspend the 0x23 service for the remainder of that
ignition cycle.
