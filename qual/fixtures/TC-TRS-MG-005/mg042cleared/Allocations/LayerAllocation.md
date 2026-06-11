---
type: Allocation
name: LayerAllocation
features:
  - name: brakeFunctionToECU
    type: Allocation
    allocatedFrom: Logical::BrakeFunction
    allocatedTo: Physical::BrakeECU
---

Routes the logical BrakeFunction to the physical BrakeECU through an Allocation
rather than a supertype edge. This is the MagicGrid-sanctioned way to cross the
logical/physical boundary, so MG042 is cleared.
