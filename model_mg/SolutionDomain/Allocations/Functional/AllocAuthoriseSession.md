---
type: Allocation
name: AllocAuthoriseSession
features:
  - name: source
    allocatedFrom: ProblemDomain::WhiteBox::FunctionalAnalysis::AuthoriseSession
    allocatedTo: ProblemDomain::WhiteBox::LogicalSubsystems::UserInterfaceSubsystem
---

Functional allocation: the **AuthoriseSession** action is performed by the
**UserInterfaceSubsystem**, which drives driver authentication at the stall.
