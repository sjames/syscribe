---
type: Requirement
id: REQ-TRS-IFACE-002
name: "W023 path-existence check shall apply to Interface and InterfaceDef"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-IFACE-000]
breakdownAdr: Decisions::InterfaceImplTraceADR
tags:
  - validation
  - traceability
---

The W023 validation rule shall fire for `Interface` and `InterfaceDef` elements that
carry an `implementedBy:` field under the same conditions that apply to `Part` and
`PartDef`:

- The element is **not** in `status: draft`.
- An `implementedBy:` entry resolves to a local path (model-relative, repo-relative,
  absolute, or `file://` URI).
- That local path does not exist on disk.

Remote URIs (HTTP/HTTPS or other non-`file://` schemes) shall not be checked and shall
not trigger W023.

Draft elements shall be exempt so that interface stubs can be defined before the
implementing file exists.
