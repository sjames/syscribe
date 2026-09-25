---
type: Configuration
id: CONF-VR-CHILD-001
name: "Child variant overriding the inherited gain"
status: draft
featureModel: Features
derivedFrom: CONF-VR-BASE-001
features:
  Features: true
parameterBindings:
  Features::Opt.gain: 4
---
Selects `Opt` only through inheritance, so its own `gain` binding is legal (no E203) only while the
inherited selection is in effect.
