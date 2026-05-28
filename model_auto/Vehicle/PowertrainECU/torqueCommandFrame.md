---
type: Item
name: Torque Command Frame
typedBy: System::Interfaces::CANFrame
---

Item flow representing the torque command CAN frame transmitted from the ThrottleControl
software component to the powertrain CAN network via the CAN transceiver. The frame
is produced every 10 ms, carrying the demanded throttle position and engine torque setpoint
to the transmission control module for gear-shift coordination.

## Frame layout

| Bytes | Field | Range | Resolution |
|---|---|---|---|
| 0–1 | Throttle position demand | 0–100 % | 0.1 % / LSB |
| 2–3 | Engine torque setpoint | −500 to +500 Nm | 0.1 Nm / LSB |
| 4 | Limiter status flags | bitmask | soft-limit, hard-limit, limp-home |
| 5 | Freshness counter (4-bit, upper nibble) | 0–15 | rolls over every 16 frames |
| 5–7 | MAC (24-bit, lower nibble of byte 5 + bytes 6–7) | — | CMAC-AES-128 truncated |

## Authentication

The 24-bit CMAC-AES-128 MAC (SC-ENG-001) covers bytes 0–4 plus the 128-bit session key
and freshness counter. The TCM verifies the MAC before acting on the torque setpoint.
A single MAC failure is logged; three consecutive failures suppress the torque setpoint
and the TCM falls back to its own last-known-good engine torque estimate.

## Latency budget

End-to-end latency from TPS reading to frame reaching TCM: ThrottleControl PID (10 ms) +
CanIf SecOC hook (< 0.5 ms) + CAN propagation at 500 kbit/s (< 0.2 ms) = ≈ 10.7 ms.
This is within the TCM shift-point decision window of 20 ms.
