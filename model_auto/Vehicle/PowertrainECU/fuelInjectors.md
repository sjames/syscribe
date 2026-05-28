---
type: Part
name: Fuel Injectors
typedBy: System::Actuators::FuelInjector
domain: hardware
multiplicity: "4"
---

Set of four port fuel injectors, one per cylinder, mounted in the intake manifold runner
directly upstream of the intake valve. Sequential injection mode is used — each injector
fires once per engine cycle (720° crank) timed to the intake stroke.

## Static flow specification

Static flow rate: 240 cc/min at 3.0 bar differential fuel rail pressure. This gives a
theoretical maximum injection duration at idle (approx. 2.5 ms at 800 rpm) and at maximum
load, both within the ECU's 16-bit timer resolution at the configured 0.5 µs tick period.

## Peak-and-hold drive circuit

Each injector is driven by a peak-and-hold circuit on the ECU power stage:
4 A peak current for 1 ms (pulls the needle off the seat), then reduced to 1 A hold
current for the remainder of the injection pulse. This minimises valve bounce and power
dissipation in the solenoid coil. The drive circuit monitors the current waveform; a
missing current peak sets DTC P0201–P0204 (injector circuit fault, respective cylinder).

## Rev limiter interaction

The `FuelControl` soft rev limiter (6200 rpm ignition retard) and hard rev limiter
(6500 rpm fuel cut) both act on these injectors. The hard fuel cut removes injection
pulses for individual cylinders in a rotating sequence to smooth the torque reduction;
REQ-ENG-SAFE-004 requires this to operate independently of the throttle position signal.
