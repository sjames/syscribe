---
id: REQ-TRS-TYPE-014
type: Requirement
title: "Tool shall recognise and validate the BindingConnector element"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise and validate the `BindingConnector` usage element.

- **SysMLv2 mapping:** `BindingConnector` maps to the SysMLv2 `binding` connector (§2.2) — an equality binding between two features. As a standalone element it declares `left:` and `right:` feature chains (the binding-connector schema of §8.4.3); the inline equivalent is the `bindingConnections:` field on a host element.
- **Identity style:** **name-based (Style B)** — identity is the path-derived qualified name (basic-name grammar). It carries **no stable id**.
- **Required vs optional fields:** `type:` is required; `name:` is optional and defaults to the filename stem. In the inline `bindingConnections:` form, `left:` and `right:` are required per §8.4.3.
- **Recognition behaviour:** a file with `type: BindingConnector` is parsed as that element type (never `Unknown`); an unknown `type:` value is `E005` ([[REQ-TRS-ELEM-002]]). Base-library supertype application follows [[REQ-TRS-ELEM-003]] (§11.4).

Related: [[REQ-TRS-ELEM-001]], [[REQ-TRS-ELEM-002]], [[REQ-TRS-ELEM-003]].

**Source:** §2.2, §8.4.3

**Acceptance criteria:** A model containing a minimal `BindingConnector` parses it at its declared type (visible in `export`) and raises no `E005` for it; a sibling file with an unrecognised `type:` value raises `E005`.
