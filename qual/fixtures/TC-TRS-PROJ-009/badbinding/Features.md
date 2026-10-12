---
type: FeatureModel
name: Features
featureTree:
  - name: Profile
    id: FEAT-P9-PROFILE
    mandatory: true
    parameters:
      - {name: delayS, type: ScalarValues::Integer, default: 3, isRequired: true, bindingTime: load}
      - {name: limit, type: ScalarValues::Integer, isRequired: true, bindingTime: load}
---
