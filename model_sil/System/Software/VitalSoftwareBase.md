---
type: PartDef
name: Vital Software Base
domain: software
isAbstract: true
silLevel: 4
satisfies:
  - REQ-SIL-SW-002
  - REQ-SIL-SW-003
---

Abstract base definition for all vital (SIL 4) software components in the interlocking. All concrete vital software component definitions specialise this type.

Common properties shared by all vital SWCs:

**Cyclic execution model.** Each vital SWC executes exactly once per scan cycle. The scan period is fixed and must not exceed 20 ms. Any overrun of the scan period constitutes a safety violation and must be detected by the RTOS watchdog, which initiates the safe state sequence.

**Safe-side failure behaviour.** Any internal error detected during a scan cycle (assertion violation, watchdog timeout, unexpected exception, or computed output outside the valid range) produces the most-restrictive output for all affected objects and reports a diagnostic event. The most-restrictive output for signals is red; for points, it is the current locked position; for level crossings, it is barriers-down.

**Formal specification.** All vital SWCs are formally specified using B-Method (classical-B or Event-B). Proof obligations are discharged using the Atelier-B proof engine. The refinement chain from the abstract machine to the generated C code must be complete and free of unproven proof obligations.

**SIL 4 qualification per EN 50128.** All vital SWCs are developed, reviewed, tested, and maintained under an EN 50128 SIL 4 software safety plan, including static analysis, structural coverage at MC/DC level, and formal proof where applicable.
