---
id: REQ-TRS-QNAME-005
type: Requirement
name: "Tool shall report a qualifiedName: identity override as unsupported (W049)"
status: draft
reqDomain: software
verificationMethod: test
---

The qualified name of every element is **purely path-derived** (§4.2, §4.5,
§11.3): the directory names and the filename stem, never a frontmatter value.
Spec §3.1 formerly also listed `qualifiedName:` as an override of the derived
qualified name; that contradicts the normative path-derived rule (and the
collision rule `E108`, which relies on the path), so the override is
**not supported** and has been removed from §3.1. `qualifiedName:` remains a
recognized field only as the target pointer of a §3.10 locale documentation
variant (a file that also sets `locale:`, REQ-TRS-PARSE-010).

The tool **shall**:

- never let `qualifiedName:` change an element's qualified name;
- report warning **`W049`** on a file that sets `qualifiedName:` without
  `locale:` when the value differs from the element's path-derived qualified
  name, naming both and stating that the field is ignored (move or rename the
  file to change the qualified name);
- raise no `W049` when the value equals the path-derived qualified name
  (redundant but harmless) or when the file is a locale variant.

**Source:** GH issue #160; spec §3.1, §4.2, §4.5, §11.3.

**Acceptance criteria:** a `PartDef` at `Arch/Pump.md` declaring
`qualifiedName: Other::Pump` keeps the qualified name `Arch::Pump` and raises
exactly one `W049`; a `PartDef` whose `qualifiedName:` equals its path-derived
name raises no `W049`.
