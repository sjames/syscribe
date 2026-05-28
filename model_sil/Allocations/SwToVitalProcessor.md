---
type: Allocation
name: Software Image to Vital Processor Allocation
allocatedFrom:
  - System::InterlockingControlSoftware
allocatedTo:
  - System::Hardware::VitalProcessor
---

The Interlocking Control Software (`System::InterlockingControlSoftware`, `domain: software`,
`isDeploymentPackage: true`) is deployed to and executes on the Vital Processor hardware
(`System::Hardware::VitalProcessor`, `domain: hardware`).

Both Channel A and Channel B vital processors execute identical software images compiled
from the same source artefact (the B-Method specification and its refinements). The software
image is qualified to EN 50128 SIL 4. Physical delivery to each channel is via a dedicated
commissioning port on the IER panel; the image cannot be updated remotely.

## Partition isolation

The deployed image uses the VxWorks RTOS partition manager to enforce hard boundaries
between the vital partition (SIL 4 SWCs) and the non-vital partition (DiagnosticMonitor).
The RTOS memory protection unit (MPU) prevents any non-vital SWC from reading or writing
the vital partition's memory regions. A boundary violation causes an immediate MPU fault,
which triggers the vital partition safe-state output sequence.

## Commissioning

At commissioning, the following configuration parameters are loaded into each channel's
non-volatile parameter store:

- Track layout (track section IDs, point IDs, signal IDs)
- Conflict matrix (derived from the geographic layout, formally proved in B-Method)
- Route table (route definitions, approach locking and overlap parameters)
- Station 1 timing parameters (`maxRoutes`, `scanPeriodMs`, `moveTimeoutMs`)
