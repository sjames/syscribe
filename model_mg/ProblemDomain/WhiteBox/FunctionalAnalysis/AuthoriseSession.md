---
type: ActionDef
name: AuthoriseSession
parameters:
  - name: credential
    typedBy: ScalarValues::String
    direction: in
  - name: authorised
    typedBy: ScalarValues::Boolean
    direction: return
refines:
  - ProblemDomain::WhiteBox::SystemRequirements::Availability
custom_fields:
  mg_cell: W2
---

Functional action: validate the driver's credential against the back-office
cloud and return whether the session is authorised.
