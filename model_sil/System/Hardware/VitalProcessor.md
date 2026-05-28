---
type: PartDef
name: Vital Processor
domain: hardware
isAbstract: true
silLevel: 4
satisfies:
  - REQ-SIL-SW-001
  - REQ-SIL-SEC-001
features:
  - name: cpuFreqMhz
    typedBy: ScalarValues::Integer
  - name: ramKb
    typedBy: ScalarValues::Integer
  - name: flashKb
    typedBy: ScalarValues::Integer
  - name: safeStateOutputs
    typedBy: ScalarValues::Integer
---

Abstract definition of one channel of the 2oo2D vital processor. Concrete instances are Channel A and Channel B. Each channel independently executes the complete interlocking vital logic and compares its output state vector with the other channel's output via a dedicated cross-comparison bus.

Any mismatch between channel outputs forces both channels to the safe state within one scan cycle (maximum 50 ms). The safe state is achieved by de-energising all vital relay outputs simultaneously, placing all signals at the most-restrictive aspect and cancelling all set routes.

The processor includes hardware watchdog circuits that independently monitor the execution of the application software on both channels. A watchdog timeout on either channel triggers the safe state relay chain on that channel and signals the partner channel to initiate its own safe state sequence.

Hardware integrity is assessed to IEC 61508 SIL 4 using FMEA and fault tree analysis. The processor board meets the diagnostic coverage and safe failure fraction requirements of EN 50129 for SIL 4 random hardware failures.
