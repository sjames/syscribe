---
type: Package
name: Integration
repoImports:
  - repo: brakes
    qname: BrakeSystem
    as: Brakes
  - repo: shared
    qname: Types
    as: Lib
---
Mounts the peer brake system at Integration::Brakes and the shared types at Integration::Lib.
