---
type: Part
name: engine
typedBy: EngDef
subsets: [./carUsage]
redefines: Sys::Engines::Engine::mass
satisfies: [REQ-FIX-001]
---
Typed through an alias, subsets a sibling via `./`, redefines an inline feature, satisfies by id.
