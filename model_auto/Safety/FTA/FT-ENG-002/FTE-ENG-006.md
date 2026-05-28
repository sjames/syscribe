---
type: FaultTreeEvent
id: FTE-ENG-006
title: CPS target wheel corrosion reducing signal amplitude
eventKind: basic
ref: System::Sensors::CrankshaftPositionSensor
failureRate: 4.0e-8
probability: 4.0e-5
---

Progressive corrosion of the crankshaft reluctor (target) wheel teeth reduces
the magnetic flux variation sensed by the CPS, lowering signal amplitude below
the ECU's adaptive detection threshold.

This is a latent failure mode that develops over the engine's service life
in high-humidity or road-salt environments. The signal is not immediately lost;
instead the amplitude margin is eroded until a secondary perturbation (such as
an EMC burst, see FTE-ENG-007) can push the signal below the decoding threshold.

In isolation, the ECU's adaptive threshold algorithm compensates for gradual
amplitude reduction and this event does not cause a stall. It becomes relevant
only as one input to the common-cause AND gate FTG-ENG-004.
