---
type: Configuration
id: CONF-BASE-001
name: "Base product tier"
status: approved
featureModel: Features
features:
  Features::CloudSync: false
---

Entry-level product build. RTH events are logged locally only —
`Planning::PI-RTH-CLOUDLOG-001` (gated on `FEAT-CLOUD-SYNC`) is excluded from
this configuration's projection.
