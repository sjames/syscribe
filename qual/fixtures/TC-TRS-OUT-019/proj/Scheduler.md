---
type: PartDef
name: Scheduler
satisfies:
  - REQ-SBOM-001
implementedBy:
  - src/scheduler/mod.rs
  - crates.io:tokio@1.38.0
  - npm:lodash@4.17.21
  - github:embedded/embedded-hal@v1.0.0
---
The scheduler subsystem.
