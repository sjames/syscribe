---
type: Requirement
id: REQ-TRS-DOCGEN-003
name: "doc build renders a Document and doc build --check reports one that is out of date"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - documents
---

`doc build <Document>… | --all [--check] [--config <C>]` shall build the documents the model declares (`REQ-TRS-DOCGEN-002`) and say when one no longer matches the model (GH #532).

1. {#build} `doc build` renders the Document's template (`REQ-TRS-DOCGEN-001`), with the variable `document` (name, qualified name, version, audience, config, status), for its `config:` or the `--config` given, and writes the rendered file. A template that does not render writes nothing and the command exits 1.
2. {#manifest} It writes `<rendered>.manifest.json` beside the rendered file: the document, the template and its hash, the Configuration, the hash of the rendered text, and each helper call the template made with a hash of what it returned.
3. {#check} `doc build --check` writes nothing. A document is out of date when a recorded call now returns something else (a `value()` is shown with what it is now) or no longer resolves, the template changed, the Configuration differs, the rendered file is missing or differs from what was built, there is no manifest, or an artefact is missing or older than the rendered file. Each reason is listed, and the command exits 2; it exits 0 when every checked document is up to date.
4. {#pdf} The command writes text. Making a PDF of it is left to the tool that owns that step.
