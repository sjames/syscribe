---
type: View
name: RequirementList
expose: [{target: "Requirements::*", filter: "type == Requirement"}]
columns:
  - {field: ref, heading: Id}
  - {field: name, heading: Requirement}
  - {field: status, heading: Status}
  - {field: satisfiedBy, heading: Satisfied by, empty: "—"}
sort: [ref]
caption: "Requirements"
---

The requirements, with the elements that satisfy each.
