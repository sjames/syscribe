---
type: Configuration
id: CONF-HIGHER-001
name: "Higher tier — binds the colliding key to an out-of-range value"
status: approved
featureModel: Features
features:
  Features: true
  Features::TopSecret: true
parameterBindings:
  Features::TopSecret.forbidden: 999
---
999 would trip the lower tier's voltage range check (0..10) if bindTo could somehow see across
the repo boundary — it must not. `[repos.battery]` (../lower) is declared but never walked by
`feature-check` — this model's own elements never contain the lower tier's bindTo/range metadata.
