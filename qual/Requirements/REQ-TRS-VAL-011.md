---
id: REQ-TRS-VAL-011
type: Requirement
name: Actionable Findings and Gherkin Scaffolding
title: Tool shall emit actionable E106/W701 messages and scaffold Gherkin scenarios
status: draft
reqDomain: software
verificationMethod: test
---

To reduce authoring friction while preserving the strong 1:1 `testFunctions[].scenario` ↔ `Scenario:` invariant, the tool **shall**:

- emit an **actionable** `E106` message that includes the exact line to add (`Scenario: <title>`) and the command to fix it;
- emit an **actionable** `W701` message that includes the exact frontmatter line to add (`verificationMethod: test`);
- provide a `scaffold-gherkin <TC>` command that prints stub `Scenario:` blocks for every `testFunctions[].scenario`, marking which already exist and which are missing;
- support `scaffold-gherkin <TC> --fix`, which inserts the missing `Scenario:` blocks into the TestCase file's first `gherkin` block (creating one with a `Feature:` line when absent) so that `E106` subsequently passes.

**Source:** Issue #5 (Gherkin↔testFunction scaffolding + actionable E106/W701 hints)

**Acceptance criteria:** the `E106` message contains `Scenario: <title>`; running `scaffold-gherkin <TC> --fix` on a TestCase with a missing scenario inserts it and clears the `E106` finding.
