---
type: PartDef
name: Interlocking Control Software
domain: software
isDeploymentPackage: true
---

The Interlocking Control Software is the deployable software image that is loaded onto each vital processor channel. It contains all vital (SIL 4) and non-vital software components (SWCs) required to operate the interlocking.

The image is structured as a set of RTOS partitions: the vital partition hosts the RouteProcessor, ConflictChecker, PointsController, SignalController, and SafetyCommLayer SWCs; the non-vital partition hosts the DiagnosticMonitor. Strict memory and time partitioning (per IEC 62443 and EN 50128 SIL 4 requirements) ensures that a failure in the non-vital partition cannot propagate to the vital partition.

Both channel A and channel B run identical software images compiled from the same source artefact, but may run on diverse compiler toolchains or hardware configurations to achieve software diversity. The image is qualified to EN 50128 SIL 4 through a full V-cycle including formal specification in B-Method, proof obligation discharge, and systematic testing.
