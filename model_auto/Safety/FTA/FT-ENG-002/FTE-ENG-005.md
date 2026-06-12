---
type: FaultTreeEvent
id: FTE-ENG-005
name: ECU supply voltage dropout below 7 V
eventKind: basic
ref: System::EngineECU
failureRate: 8.0e-8
probability: 8.0e-5
---

A transient or sustained dropout of the ECU supply voltage below 7 V causes
the microcontroller to reset or enter an undefined state, resulting in loss of
crankshaft position signal processing and engine stall.

Causes include alternator load-dump, battery connection intermittency,
or excessive inrush current from simultaneous high-load accessories. The ECU
supply line includes a bulk capacitor providing hold-up for transients up to
50 ms, so only sustained dropouts trigger this event.

The ECU supply voltage is monitored by the on-chip ADC; a brownout detect
circuit below 6.5 V asserts a controlled reset rather than allowing undefined
microcontroller behaviour. This detection mechanism reduces the probability
of an uncontrolled stall relative to a hard power loss.
