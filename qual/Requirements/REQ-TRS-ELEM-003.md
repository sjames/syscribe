---
id: REQ-TRS-ELEM-003
type: Requirement
title: "Tool shall apply implicit base library supertypes when no supertype: is given"
status: draft
reqDomain: software
verificationMethod: analysis
---

The tool **shall** apply the implicit supertype rules defined in §11.4 to definition element types that carry no explicit `supertype:` field. When `supertype:` is explicitly set, it **shall** replace (not supplement) the implicit default.

**Source:** §11.4

**Acceptance criteria:** A `PartDef` with no `supertype:` is treated as specializing `Parts::Part` during graph construction; adding `supertype: MyBase` removes the implicit default.
