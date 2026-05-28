---
type: Part
name: Crank Sensor
typedBy: System::Sensors::CrankshaftPositionSensor
domain: hardware
subsets: [System::EngineECU::primarySpeedSensor]
---

Crankshaft position sensor instance mounted on the engine block, reading a 60-2 tooth
wheel pressed onto the crankshaft front hub. The 60-2 pattern (60 teeth with 2 missing)
provides a unique reference gap used to establish absolute crank angle at every revolution.

## Signal characteristics

The sensor is a Hall-effect type producing a push-pull digital square wave at 12 V logic
level, referenced to sensor ground on the ECU side. Signal frequency ranges from ~15 Hz at
starter-motor cranking speed (50 rpm) to approximately 2 kHz at maximum engine speed
(6500 rpm). Signal conditioning (hysteresis comparator) is performed on the ECU input stage.

## Stall detection role

The `EngineStallMonitor` SWC monitors inter-tooth pulse intervals. Loss of three consecutive
tooth pulses triggers a stall warning (REQ-ENG-SAFE-003). A harness open-circuit is the
dominant failure mode (FTE-ENG-004, λ = 1.2 × 10⁻⁶/h), accounting for 93 % of the total
CPS signal-loss probability in FT-ENG-002.

## Environmental rating

AEC-Q100 Grade 0 (−40 °C to +150 °C); vibration resistance per USCAR-21, Class D (engine
block mounting location). Connector: 3-pin Bosch Compact 1.5 sealed.
