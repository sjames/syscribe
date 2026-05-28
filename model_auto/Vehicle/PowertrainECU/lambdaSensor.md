---
type: Part
name: Lambda Sensor
typedBy: System::Sensors::LambdaSensor
domain: hardware
---

Wideband lambda (UEGO — Universal Exhaust Gas Oxygen) sensor mounted in the exhaust manifold
collector, upstream of the three-way catalytic converter. Provides a continuous linear output
current representing the air–fuel ratio from λ = 0.7 (rich) to λ = 1.65 (lean).

## Heater pre-conditioning

The sensor contains an integrated ceramic heater drawing approximately 7 W during the
pre-conditioning phase. The FuelControl SWC runs open-loop (base fuel map only) until the
heater element reaches operating temperature (≥ 300 °C), indicated by the sensor reaching
its target cell voltage. Pre-conditioning typically completes within 20–30 seconds from cold
start at ambient temperatures above 0 °C.

## Closed-loop operation

Once operational, the lambda signal drives the closed-loop PI corrector in `FuelControl`.
At steady state the lambda correction factor is constrained to ±25 % of the base injection
pulse width to detect sensor saturation or fuel system drift. Corrections beyond this range
set DTC P0171 (lean) or P0172 (rich) and revert to open-loop operation.

## Location note

Mounting in the manifold collector (pre-catalyst) gives the fastest response to combustion
events but exposes the sensor to higher thermal and mechanical stress than a post-catalyst
location. The λ = 1.00 stoichiometric target of REQ-ENG-PERF-002 refers to the pre-catalyst
measurement point.
