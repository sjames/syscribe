---
type: PartDef
name: MpptArray
features:
  - {name: tracker1, typedBy: Behavior::Parts::MpptTrackerPart}
  - {name: tracker2, typedBy: Behavior::Parts::MpptTrackerPart}
  - {name: tracker3, typedBy: Behavior::Parts::MpptTrackerPart}
---

The trackers of a three-input product: one independent instance per boost stage (REQ-GTI-SYS-020, REQ-GTI-SW-006). A two-input product runs the first two.
