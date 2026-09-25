---
id: REQ-TRS-QNAME-002
type: Requirement
name: "Package qualified-name segment shall be the directory name; _index.md name: is a label only"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** use a directory's name on disk as that directory's namespace segment in all qualified names. A `name:` field in the directory's `_index.md` **shall** be treated as a display label only and **shall not** change any qualified name.

**Source:** §11.3 step 2; §4.2

**Acceptance criteria:** If `model/VehicleSystem/_index.md` contains `name: VS`, then `model/VehicleSystem/Engine.md` has qualified name `VehicleSystem::Engine`, not `VS::Engine`.
