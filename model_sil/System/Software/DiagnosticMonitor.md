---
type: PartDef
name: Diagnostic Monitor
domain: software
---

The Diagnostic Monitor is a non-vital software component that collects and presents health and status information for the maintainer workstation. It has read-only access to data published by the vital SWCs through a one-way data channel; it has no write access to any vital data structure.

The component collects:

- CPU load and scan cycle utilisation for each channel
- Memory usage (heap fragmentation, stack high watermarks) for each partition
- Communication latency and error counters for the inter-channel bus and the field bus
- Field equipment status: relay contact states, track circuit voltages, points detection status, signal lamp feedback
- Vital event log: route setting requests, approvals, cancellations, and failure events with timestamps

Data is forwarded to the maintainer workstation over a separate non-vital network port. The maintainer workstation has no control authority over the interlocking; it can only display information.

The DiagnosticMonitor runs in a separate RTOS partition from the vital SWCs, partitioned by time (CPU quota) and space (memory regions). The partition boundary is enforced by the RTOS memory protection unit (MPU). A software fault, infinite loop, or exception within the DiagnosticMonitor partition cannot affect the vital partition's scan cycle timing or data integrity.

A failure in the DiagnosticMonitor is detected by its own partition watchdog and reported by the RTOS to the vital processor's health monitor. The vital processor logs the event but does not enter the safe state as a result — the DiagnosticMonitor is a non-vital component with no influence on safety outputs.
