---
type: Zone
id: ZN-ENG-002
name: Diagnostic Zone
status: approved
targetSL: 2
achievedSL: 2
members:
  - System::Software::DiagnosticSecurityLayer
  - System::Software::SecureBootManager
rationale: The OBD-II reflash and calibration path; reachable by a technician with physical access.
---

The zone holding the UDS session handling and the secure boot chain.
