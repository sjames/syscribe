---
id: REQ-TRS-VAL-001
type: Requirement
title: Tool shall enforce all parse-time error rules E001–E015 and E300–E304
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** enforce every parse-time error rule in the following table, as defined in §11.12 of the Syscribe format specification. Each rule **shall** be emitted at parse time (while reading the individual file that triggers it).

| Code | Condition |
|---|---|
| `E001` | File does not begin with `---` |
| `E002` | Frontmatter is not valid YAML 1.2 |
| `E004` | A required field is absent |
| `E005` | `type:` value is not in the element type inventory |
| `E006` | `id:` present but does not match the required pattern |
| `E007` | `status:` value is not in the allowed enum |
| `E008` | `testLevel:` value is not in `L1`–`L5` |
| `E009` | `silLevel:` value is not an integer in 1–4 |
| `E010` | `asilLevel:` value is not in `A`–`D` |
| `E011` | Native `TestCase` body has no ` ```gherkin ` fenced block |
| `E012` | Native `Requirement` body has no normative text |
| `E013` | `verifies:` list is present but empty |
| `E014` | `Scenario Outline:` block has no `Examples:` table |
| `E015` | First Gherkin block has no `Feature:` line |
| `E300` | `ADR.id` does not match the `ADR-*` pattern |
| `E301` | `ADR` is missing a required field |
| `E302` | `reqDomain:` value is not `system`, `hardware`, or `software` |
| `E303` | `domain:` value is not `system`, `hardware`, or `software` |
| `E304` | `ADR.status` is not in the allowed enum |

**Source:** §11.12 (parse-time errors)

**Acceptance criteria:** For each code, a crafted model file that triggers exactly that condition produces exactly one finding with that code.
