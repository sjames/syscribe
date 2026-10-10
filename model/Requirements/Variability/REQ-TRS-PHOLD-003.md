---
type: Requirement
id: REQ-TRS-PHOLD-003
name: "Placeholder consumers and resolved values are discoverable"
status: draft
reqDomain: software
reqClass: system
tags:
  - variability
---

An author shall be able to see which elements use a feature parameter and what value it takes in each configuration (GH #267).

## Behavior

- `feature <qname>` lists, per parameter, the elements whose body or name references it, and the value it resolves to in every configuration that selects the feature (binding, else fixed value, else default, else unbound). Text and `--json`.
- `links <element>` lists an element's placeholder references as outbound `placeholder` relationships to the referenced feature, and `refs <feature>` lists the elements that reference it.
