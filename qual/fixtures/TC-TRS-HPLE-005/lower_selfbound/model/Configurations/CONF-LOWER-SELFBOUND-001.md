---
type: Configuration
id: CONF-LOWER-SELFBOUND-001
name: "Lower tier — binds the same key its own bindTo names (positive control)"
status: approved
featureModel: Features
features:
  Features: true
  Features::Cell: true
parameterBindings:
  Features::TopSecret.forbidden: 99
---
99 is outside voltage's declared 0..10 range — this is a genuinely local match, expected to raise
a propagation-range finding. (The unresolved-FeatureDef finding this also raises is expected too —
`Features::TopSecret` genuinely doesn't exist in this model; that's the whole point.)
