---
type: Part
name: Stall Monitor
typedBy: System::Software::EngineStallMonitor
domain: software
---

EngineStallMonitor AUTOSAR SWC instance running in the 20 ms StallMonitorTask (ASIL B
partition). Monitors CPS tooth-pulse intervals to detect impending or actual engine stall
at vehicle speed and initiates a controlled deceleration sequence.

## Stall detection algorithm

At each 20 ms task activation, the monitor computes the instantaneous engine speed from the
last four CPS tooth intervals (rolling average to reduce noise). If:
- Speed drops below 400 rpm **and** vehicle speed (from transmission CAN message) is > 20 km/h
  (indicating the engine would stall the drivetrain): warning issued at T = 0.
- Speed drops below 200 rpm: stall declared; controlled deceleration sequence activated.

## Controlled deceleration sequence

1. Transmission shift request (CAN message) to downshift one gear.
2. Throttle demand set to idle if PPS < 10 %.
3. DTC P0316 (misfires detected during engine warm-up period) set if applicable.
4. Warning lamp illuminated; driver notified via instrument cluster CAN message.

The sequence must complete within the 500 ms FTTI for SG-ENG-002. At 20 ms task cycle with
< 10 ms CAN scheduling latency, the end-to-end latency is well within budget.
