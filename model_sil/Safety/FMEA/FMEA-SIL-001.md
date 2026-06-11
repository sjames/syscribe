---
id: FMEA-SIL-001
type: FMEASheet
title: FMEA — Vital Processor and Safety Communication
status: approved
entries:
  - id: FM-SIL-001
    ref: System::Hardware::VitalProcessor
    failureMode: CPU computation error — incorrect interlocking output calculated
    effect: One channel produces wrong route or signal state
    cause: Single-event upset (SEU) in processor core or memory
    fmeaSeverity: 9
    occurrence: 2
    detection: 1
    recommendedAction: 2oo2 cross-comparison detects within one scan cycle; ECC memory for SEU mitigation

  - id: FM-SIL-002
    ref: System::Hardware::VitalProcessor
    failureMode: Inter-channel comparison bus failure — no comparison possible
    effect: Interlocking defaults to safe state (fail-safe)
    cause: CAN/Ethernet physical layer failure on cross-comparison link
    fmeaSeverity: 2
    occurrence: 2
    detection: 1
    recommendedAction: Loss of comparison immediately triggers safe state — this is a safe-side failure; DTC raised for maintainer

  - id: FM-SIL-003
    ref: System::Hardware::VitalProcessor
    failureMode: Power supply undervoltage — processor resets mid-cycle
    effect: Incomplete scan cycle; outputs held in last state by fail-safe relay until next valid cycle
    cause: Input supply transient or reverse polarity
    fmeaSeverity: 4
    occurrence: 2
    detection: 2
    recommendedAction: Wide-range power supply module with brownout detection; watchdog enforces safe-state on reset

  - id: FM-SIL-004
    ref: System::Software::SafetyCommLayer
    failureMode: Safety code CRC mismatch on field bus — corrupted command
    effect: Object controller rejects message; actuator held in current position
    cause: EMC-induced bit flip on field bus
    fmeaSeverity: 3
    occurrence: 3
    detection: 1
    recommendedAction: EN 50159 CRC32 detects all burst errors up to 16 bits; message rejected, not acted upon; sequence number prevents replay of earlier valid command

  - id: FM-SIL-005
    ref: System::Software::SafetyCommLayer
    failureMode: Sequence number wrap-around not detected — old message replayed
    effect: Stale command actioned, potentially reversing a recently-changed state
    cause: Counter wrap in 16-bit field with short message intervals
    fmeaSeverity: 6
    occurrence: 1
    detection: 2
    recommendedAction: Use 32-bit sequence number (wrap period >> system lifetime); timestamp cross-check within defined time window

  - id: FM-SIL-006
    ref: System::Software::RouteProcessor
    failureMode: Route table corruption — incorrect route-to-section mapping
    effect: Wrong sections locked or released for a route; potential for a conflict to be undetected
    cause: SRAM bit-flip under radiation or SEU
    fmeaSeverity: 9
    occurrence: 1
    detection: 2
    recommendedAction: Route table in ECC-protected memory; CRC of table checked on every scan cycle; mismatch triggers safe state

  - id: FM-SIL-007
    ref: System::Software::ConflictChecker
    failureMode: Conflict matrix bit cleared — two conflicting routes no longer detected as conflicting
    effect: Both routes set simultaneously without detection
    cause: Memory corruption or software defect
    fmeaSeverity: 10
    occurrence: 1
    detection: 1
    recommendedAction: Conflict matrix is read-only after initialisation, stored in flash; CRC checked on every scan; diversity — channel B independently holds and checks its own copy

  - id: FM-SIL-008
    ref: System::Software::DiagnosticMonitor
    failureMode: Diagnostic monitor writes to vital memory partition
    effect: Vital data corrupted — interlocking produces incorrect output
    cause: Software error in non-vital SWC crosses partition boundary
    fmeaSeverity: 9
    occurrence: 1
    detection: 2
    recommendedAction: RTOS memory protection unit (MPU) enforces hard partition boundary; any write attempt from DiagnosticMonitor to vital address space causes MPU fault and vital safe-state assertion
---

Scope: vital processor hardware (VitalProcessor), safety communication layer (SafetyCommLayer), and vital application software (RouteProcessor, ConflictChecker). Includes the non-vital DiagnosticMonitor as a potential source of partition crossing failures. FMEA conducted per IEC 60812.

**Severity scale:** 1 = no effect on safety or operation; 10 = catastrophic — direct cause of a SIL 4 hazardous event without any independent mitigation.

**Occurrence scale:** 1 = extremely unlikely (< 10⁻⁹ /h); 10 = very likely (> 10⁻³ /h).

**Detection scale:** 1 = near-certain detection before effect occurs; 10 = undetectable by any means in the system.

**RPN threshold.** Entries with Risk Priority Number (RPN = Severity × Occurrence × Detection) > 100 require formal mitigation documented in the design description. FM-SIL-001 (RPN = 18), FM-SIL-007 (RPN = 10), and FM-SIL-008 (RPN = 18) all have mitigations already described; none exceed 100 because of the high-detection effectiveness of the 2oo2 architecture and scan-cycle CRC checking.

**Key finding.** The conflict matrix (FM-SIL-007) has the highest severity (10) — a single-bit corruption directly leads to the SG-SIL-001 top event. The combination of read-only storage, per-cycle CRC, and channel diversity reduces the occurrence and detection ratings to their minimum values, giving an RPN of 10. This entry should be revisited if any change is made to the conflict matrix storage architecture.
