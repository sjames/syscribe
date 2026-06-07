---
type: FeatureDef
name: Base
groupKind: mandatory
requires:
  - Features::Propulsion
  - Features::Payload
  - Features::DataLink
---

The common base shared by every product in the line. Mandatory (present in every
configuration); it requires a selection from each mandatory feature group —
propulsion, payload, and data link — so that no product can omit them.
