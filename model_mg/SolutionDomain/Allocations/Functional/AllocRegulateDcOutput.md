---
type: Allocation
name: AllocRegulateDcOutput
features:
  - name: source
    type: Allocation
    allocatedFrom: ProblemDomain::WhiteBox::FunctionalAnalysis::RegulateDcOutput
    allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::PowerConversionSubsystem
---

Functional allocation: the **RegulateDcOutput** action is performed by the
**PowerConversionSubsystem**, which closes the current/voltage control loop.
