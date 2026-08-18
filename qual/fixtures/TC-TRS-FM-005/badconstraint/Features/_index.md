---
type: FeatureModel
name: Features
featureTree:
  - name: Wdt
    id: FEAT-FM5-BC-WDT
    groupKind: optional
crossTreeConstraints:
  - feature: DoesNotExist
    requires: [Wdt]
---

`crossTreeConstraints: feature:` references a feature not defined in this
sheet's own `featureTree:` — nothing local to attach the constraint to.
