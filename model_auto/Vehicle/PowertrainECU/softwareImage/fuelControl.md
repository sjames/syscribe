---
type: Part
name: Fuel Control
typedBy: System::Software::FuelControl
domain: software
---

FuelControl AUTOSAR SWC instance running in the 10 ms FuelControlTask (QM partition).
Manages sequential port injection, lambda closed-loop air–fuel ratio control, cold-start
enrichment, and the software rev limiter.

## Base fuel calculation

Injection pulse width is computed from a 16 × 16 speed/load map (rpm × intake manifold
pressure). Map values are stored in the calibration flash sector and are write-protected
during normal operation (SC-ENG-002 programming access required to modify calibration).

## Lambda closed-loop

At operating temperature (lambda sensor heater pre-condition complete), a PI corrector
trims the base injection pulse toward λ = 1.00 ± 0.02. The I-term is reset on deceleration
fuel cut and on significant load steps to prevent windup causing the overshoot fault mode
`FM-ENG-005` in FMEA-ENG-001.

## Rev limiter

Soft limit at 6200 rpm: ignition timing retarded progressively (up to −20° BTDC) to reduce
torque and discourage sustained high-rpm operation. Hard limit at 6500 rpm: injection pulses
withheld in a rotating-cylinder sequence (to avoid single-cylinder heat concentration) until
engine speed falls below 6400 rpm (250 rpm hysteresis). This logic operates on the raw CPS
tooth-count signal and is explicitly independent of TPS reading (REQ-ENG-SAFE-004).
