---
type: Requirement
id: REQ-TRS-DOCGEN-002
name: "A Document element declares a document built from a template"
status: approved
reqDomain: software
reqClass: system
tags:
  - format
  - documents
---

The format shall have a `Document` element type (GH #532), name-identified, that makes a document built from a template part of the model.

1. {#fields} `template:` (required) names the template by a source path resolved like a TestCase's `sourceFile`. `rendered:` names where the rendered text is written (default: beside the template, its name without the template suffix). `artefacts:` (a path or a list) names what a further tool makes of it. `version:`, `audience:` and `config:` (a `Configuration`) are optional.
2. {#e508} A declaration with no `template:`, a field of the wrong shape, or a `config:` that does not resolve to a Configuration is error `E508`.
3. {#w505} A non-draft Document whose template file does not exist is warning `W505`.
4. {#keys} The declaration keys are schema fields on a `Document` (no `W047`) and stay unrecognised on any other type.
5. {#discover} `template Document`, `help Document` and `doc list [--json]` describe and list Documents.
