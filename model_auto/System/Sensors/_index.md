---
type: Package
name: Sensors
---

Input sensor PartDefs connected to the Engine ECU. All sensors specialise the abstract
`Sensor` base, which defines the common interface attributes: `signalOutputType`,
`operatingTempMin`, and `operatingTempMax`.

## Sensor inventory

| PartDef | Signal type | Key parameter | Safety relevance |
|---|---|---|---|
| `Sensor` (abstract) | — | — | Base type; not instantiated directly |
| `CrankshaftPositionSensor` | Digital (Hall pulse) | `teethCount: 60-2` | Stall detection (ASIL B, REQ-ENG-SAFE-003) |
| `ThrottlePositionSensor` | Analog voltage (dual-track) | `supplyVoltage: 5 V` | Unintended acceleration detection (ASIL D, REQ-ENG-SAFE-001/005) |
| `LambdaSensor` | Analog (UEGO pump current) | `heatingPower: 7 W` | Fuel trim, emissions compliance (REQ-ENG-PERF-002) |

## Dual-track redundancy

The `ThrottlePositionSensor` is the only sensor with a redundant signal path. Track 1
and track 2 are ratiometric (track 2 ≈ 5 V − track 1) so any harness short or sensor
element failure causes divergence detectable by the `SafetyMonitor` within one 5 ms cycle.
The 5 % divergence threshold is configured in the SafetyMonitor SWC deployed instance.
