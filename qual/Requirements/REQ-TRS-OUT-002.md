---
id: REQ-TRS-OUT-002
type: Requirement
title: Report shall list each finding with severity, code, element reference, and description
status: draft
reqDomain: software
verificationMethod: test
---

The tool's output report **shall** include, for each finding: (a) severity (`Error` or `Warning`), (b) rule code (e.g. `E001`), (c) element qualified name or file path, (d) a human-readable description of the violation.

**Source:** §11.7

**Acceptance criteria:** Each finding line is parseable to extract all four fields without ambiguity.
