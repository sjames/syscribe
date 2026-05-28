---
type: Interface
name: Powertrain CAN Bus
typedBy: System::Interfaces::PowertrainCANInterface
---

Powertrain CAN network interface instance on the PowertrainECU. The physical medium is
a 120 Ω impedance twisted-pair backbone shared with the transmission control module (TCM),
brake control module (ABS/ESP), and body control module (BCM) on this vehicle variant.

## Bus topology

Four nodes on the powertrain CAN segment: ECU (this), TCM, ABS/ESP module, and gateway ECU
(bridges to chassis CAN and infotainment bus). Bus length: approximately 2.5 m point-to-point
with two 150 mm stubs to off-backbone nodes. All nodes operate at 500 kbit/s; the gateway
bridges at this rate on both sides.

## Safety-critical PDUs

The following PDUs on this bus carry SecOC MACs per SC-ENG-001 and are monitored for
authentication failures by the `CANSecurityModule`:

| PDU | Arbitration ID | Cycle | Producer |
|---|---|---|---|
| Torque command | 0x0C8 | 10 ms | Engine ECU |
| Transmission shift request | 0x0C9 | 20 ms | Engine ECU |
| Brake torque coordination | 0x0CA | 10 ms | Engine ECU |
| Wheel speed (×4) | 0x1A0–0x1A3 | 10 ms | ABS/ESP module |

## Bus load

At 500 kbit/s, the estimated bus load from all powertrain segment nodes is approximately
35 % under nominal engine management conditions, well below the 40 % guideline for
deterministic latency. Peak load during simultaneous fault-injection test scenarios
(TC-ENG-SAFE-001) reaches approximately 42 %.
