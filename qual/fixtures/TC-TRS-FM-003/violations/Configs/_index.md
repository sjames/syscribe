---
type: Package
name: Configs
parameterConstraints:
  - id: PC-GHOST-001
    expression: "Features::Sys.ghost > 0"
    severity: error
  - id: PC-UNUSED-001
    expression: "Features::Sys.sysKv > 0"
    appliesWhen:
      - Features::Unused
---
Configs package.
