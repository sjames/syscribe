---
id: FEAT-PLF-CAMERA
type: FeatureDef
name: Camera
requires: [Features::Platform::CortexM]
excludes: [Features::Link::Cellular]
bindingTime: compile
latestBindingTime: load
---

Camera payload: needs the Cortex-M platform, excludes the cellular link.
