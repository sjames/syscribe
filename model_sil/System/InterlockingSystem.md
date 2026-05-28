---
type: PartDef
name: Interlocking System
domain: hardware
features:
  - name: processorChannels
    typedBy: ScalarValues::Integer
  - name: fieldBusBaudRate
    typedBy: ScalarValues::Integer
    unit: bps
  - name: missionTime
    typedBy: ScalarValues::Real
    unit: h
---

The Interlocking System is the top-level hardware platform for the Computer-Based Interlocking (CBI). It implements a 2oo2D (two-out-of-two with diagnostics) architecture consisting of two diverse vital processor boards (channel A and channel B), each independently executing the complete interlocking vital logic.

The shared-nothing architecture ensures that neither channel can influence the other's internal processing. A cross-comparison bus allows both channels to exchange output state vectors at the end of every scan cycle. Any disagreement between the two channels' output vectors is treated as a dangerous failure and both channels are driven simultaneously to the safe state: all signals return to the most-restrictive aspect (red) and all set routes are cancelled.

Safe-state relay outputs are hardware-enforced: the relay coils are de-energised by removing power rather than by an active command, ensuring that a processor failure cannot hold a relay in the energised (permissive) position. Watchdog cross-monitoring allows each channel to monitor the other's health independently of the application software.

Hardware is certified to EN 50129 and designed to achieve SIL 4 integrity under IEC 61508 failure-rate analysis.
