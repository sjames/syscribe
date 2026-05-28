---
type: FaultTreeEvent
id: FTE-ENG-007
title: High-intensity radiated field causing CPS signal dropout
eventKind: basic
ref: System::Sensors::CrankshaftPositionSensor
failureRate: 2.0e-8
probability: 2.0e-5
---

A high-intensity radiated field (HIRF) event — from an external source such as
a roadside radar installation, radio transmitter mast, or military broadcast
facility — induces sufficient common-mode noise on the CPS signal pair to
momentarily suppress the differential signal below the ECU decoding threshold.

In a vehicle with undegraded CPS signal amplitude, the shielded harness and
differential signal conditioning provide adequate rejection of the HIRF level
specified by ISO 11452. This event applies only when the CPS signal amplitude is
already reduced (combined with FTE-ENG-006 in the AND gate FTG-ENG-004).

The failure rate reflects the statistical exposure to HIRF environments above the
ISO 11452 immunity level during normal vehicle operation, weighted by the
proportion of time spent in high-field environments (estimated < 0.1 % of vehicle
operating hours).
