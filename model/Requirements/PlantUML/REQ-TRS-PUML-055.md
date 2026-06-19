---
id: REQ-TRS-PUML-055
name: "base_url in .syscribe.toml controls shape hyperlinks in generated PlantUML"
type: Requirement
status: draft
reqClass: system
reqDomain: software
derivedFrom: REQ-TRS-PUML-020
breakdownAdr: Decisions::PlantUMLADR
tags: [diagram, plantuml, github]
---

`syscribe plantuml` shall generate `[[URL]]` shape annotations using the format:

```
<base_url>/<qref_as_path>.md
```

where `qref_as_path` is the element's qualified name with every `::` replaced by `/`.

When `[plantuml] base_url` is not configured, no `[[URL]]` annotations are emitted
(links suppressed). An explicit empty string (`base_url = ""`) also suppresses links.

Example: `base_url = "https://github.com/sjames/syscribe/blob/main/model"` and shape
`UAV::UAVSystem` produces `[[https://github.com/sjames/syscribe/blob/main/model/UAV/UAVSystem.md]]`,
which is a valid GitHub file URL.
