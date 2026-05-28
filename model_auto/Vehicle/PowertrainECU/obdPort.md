---
type: Port
name: OBD-II Port
typedBy: System::Interfaces::OBDIIPort
---

OBD-II diagnostic connector routed to the vehicle passenger compartment. The connector
is a standard 16-pin ISO 15031-3 (SAE J1962 Type A) D-shell, mounted within reach of the
driver's seat per emissions legislation requirements (< 600 mm from steering column).

## Supported services

- **UDS over CAN (ISO 14229-3 / ISO 15765-4):** ECU diagnostic and programming access.
  CAN physical address: ECU = 0x7E8 (functional address 0x7DF accepted).
- **OBD Mode $01–$09:** Standard emissions readiness PIDs (REQ-ENG-PERF-002 coverage).
- **UDS DiagnosticSessionControl 0x10:** Default session (no auth), Extended session
  (no auth), Programming session (SC-ENG-002 security access required).

## Security controls active on this port

| Control | ID | Protects |
|---|---|---|
| Seed-and-key authentication | SC-ENG-002 | Programming and extended memory access |
| ECDSA firmware signature | SC-ENG-003 | Flash reprogramming |
| Memory read access control + audit log | SC-ENG-004 | `readMemoryByAddress` service |

Physical access to this port by an unauthenticated actor is the primary threat vector
identified in `Security/TARA-ENG-001` (TS-ENG-002, TS-ENG-003, TS-ENG-004).
