---
id: REQ-W009-TRIG-001
type: Requirement
title: "Renamed or deleted tests are detected"
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** emit W009 when a `testFunctions[].function` no longer resolves
in an existing `sourceFile`.
