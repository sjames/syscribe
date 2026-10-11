---
type: PartDef
name: Secure Update
domain: software
satisfies:
  - REQ-GTI-SYS-054
features:
  - {name: image, type: Port, direction: in}
---

Bootloader with signature verification and anti-rollback counter.
