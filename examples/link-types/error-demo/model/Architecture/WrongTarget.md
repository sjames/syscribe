---
type: PartDef
name: WrongTarget
domain: hardware
links:
  mitigates: Architecture::TypoLink    # E634: target is a PartDef, targetTypes = [Requirement]
---

Points a `mitigates` link at an element of a type the link type does not permit.
