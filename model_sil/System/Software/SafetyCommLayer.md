---
type: PartDef
name: Safety Comm Layer
domain: software
silLevel: 4
supertype: System::Software::VitalSoftwareBase
satisfies:
  - REQ-SIL-SW-004
  - REQ-SIL-SEC-002
features:
  - name: sequenceNumberBits
    typedBy: ScalarValues::Integer
  - name: crcPolynomialDegree
    typedBy: ScalarValues::Integer
  - name: timeoutMs
    typedBy: ScalarValues::Integer
---

The Safety Comm Layer implements the EN 50159 Category 2 safety communication protocol over both the inter-channel cross-comparison bus (between channel A and channel B) and the field bus to the Object Controllers.

Each message transmitted by the SafetyCommLayer carries the following safety codes:

- **32-bit CRC** computed using a polynomial of degree crcPolynomialDegree, providing detection of all burst errors up to 32 bits and residual error probability below 10^-9 per message.
- **16-bit sequence number** incrementing monotonically within each communication session, detecting out-of-order delivery, repetition, and deletion.
- **Timestamp** derived from the vital processor's cycle counter, detecting stale messages beyond the defined safety time window.
- **Connection identifier** unique to each communication session, detecting masquerading and addressing errors.

The layer detects all seven EN 50159 failure categories within the defined safety time window:

| Failure category | Detection mechanism |
|---|---|
| Corruption | CRC |
| Unintended repetition | Sequence number |
| Incorrect sequence | Sequence number |
| Loss | Sequence number + timeout |
| Unacceptable delay | Timestamp + timeout |
| Insertion | Connection ID + CRC |
| Masquerading | Connection ID |

Any communication error detected by the SafetyCommLayer causes both channels to revert to the safe state within the timeoutMs window. Communication errors are logged to the DiagnosticMonitor but the safe state action is taken before logging to avoid timing dependencies on the non-vital partition.
