---
id: REQ-TRS-QNAME-002
type: Requirement
name: "Package name shall use name: from _index.md when present"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** use the `name:` field from a directory's `_index.md` file as the namespace segment for that directory in all qualified names, overriding the directory name on disk.

**Source:** §11.3 ¶2

**Acceptance criteria:** If `model/VehicleSystem/_index.md` contains `name: VS`, then `model/VehicleSystem/Engine.md` has qualified name `VS::Engine`, not `VehicleSystem::Engine`.
