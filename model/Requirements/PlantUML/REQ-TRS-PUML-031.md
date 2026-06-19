---
type: Requirement
id: REQ-TRS-PUML-031
name: "W414: pumlMode companion .puml file not yet generated"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
  - validation
---

When `pumlMode: companion` is set on a `Diagram` element, the validator shall emit
warning **W414** if the resolved `.puml` companion file does not exist on disk.

## Resolved path

The companion file path is determined by the `pumlFile:` field when present; otherwise it
defaults to `<stem>.puml` in the same directory as the `.md` file (per REQ-TRS-PUML-001).
The existence check is performed against this resolved path.

## Warning message

```
companion `.puml` file '<path>' not found — run `syscribe plantuml` to generate it
```

where `<path>` is the resolved companion file path relative to the model root.

## Severity rationale

This is a warning (not an error) because the `.puml` file is generated output, not a
model-authored source artifact. Its absence is expected in a fresh clone of the repository
before `syscribe plantuml` has been run. A missing `.puml` file does not make the model
semantically ill-formed; it only means the generation step has not yet been executed.
Authors who want to gate CI on this can promote the warning to an error with
`--deny W414`.
