---
type: Allocation
name: Software to Interlocking Hardware Allocation
allocatedFrom: System::InterlockingControlSoftware
allocatedTo: System::InterlockingSystem
---

Allocates the Interlocking Control Software deployment package to the physical CBI hardware platform (Interlocking System). This allocation satisfies the E314 constraint: `isDeploymentPackage: true` software must be physically allocated to a hardware element.

The allocation expresses that the InterlockingControlSoftware executes on the InterlockingSystem hardware platform. In deployment, the software image is loaded onto both vital processor channels (channel A and channel B) of the hardware platform. The 2oo2D execution model means the same software image runs on both channels, with the hardware cross-comparison bus providing the voting function.

The allocation boundary is the interface between the EN 50128 software lifecycle (governing the InterlockingControlSoftware) and the EN 50129 hardware lifecycle (governing the InterlockingSystem). Integration testing at this boundary is required per EN 50129 Chapter 6 to demonstrate that the combined system meets the overall SIL 4 safety integrity claim.
