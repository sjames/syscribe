---
type: Allocation
name: AllocDetectAndIsolateFault
features:
  - name: source
    type: Allocation
    allocatedFrom: ProblemDomain::WhiteBox::FunctionalAnalysis::DetectAndIsolateFault
    allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::SafetyInterlockSubsystem
---

Functional allocation: the **DetectAndIsolateFault** action is performed by the
**SafetyInterlockSubsystem**, which monitors insulation/residual current and
opens the contactors on fault.
