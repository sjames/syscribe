---
id: REQ-TRS-VAL-005
type: Requirement
name: Each validation finding shall include rule code, element reference, and description
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** include the following information in every validation finding it emits:

- The rule code (e.g. `E001`, `W300`)
- The qualified name or stable `id:` of the offending element, or the file path for parse-time errors where no element is available
- A human-readable description of the violation

**Source:** §11.7

**Acceptance criteria:** Parsing the tool's Markdown output programmatically yields rule code, element reference, and description for every finding line.
