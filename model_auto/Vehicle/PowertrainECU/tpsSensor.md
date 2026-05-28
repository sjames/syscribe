---
type: Part
name: TPS Sensor
typedBy: System::Sensors::ThrottlePositionSensor
domain: hardware
---

Dual-track throttle position sensor integrated into the throttle body assembly. The sensor
consists of two independent resistive tracks on a shared substrate, driven from a common
5 V reference supply.

## Track configuration

Track 1 outputs 0.5 V (closed) to 4.5 V (wide-open). Track 2 is complementary:
output ≈ 5 V − track 1, nominally 4.5 V (closed) to 0.5 V (wide-open). The ECU monitors
both tracks simultaneously; divergence exceeding 5 % of full-scale triggers the ASIL D
safety fault path in `SafetyMonitor` within one 5 ms monitor cycle.

## Safety significance

The dual-track design is the primary hardware measure for detecting throttle position sensor
failures (HE-ENG-001). FTE-ENG-002 characterises the probability of both tracks failing
simultaneously (5 × 10⁻⁷/h), which is the dominant input to the AND gate FTG-ENG-002 in
FT-ENG-001.

## Physical installation

Spring-return type; the sensor wiper arm is mechanically coupled to the throttle shaft via
a press-fit bushing. IP67 rated. Connector: 6-pin Tyco MCP 2.8.
