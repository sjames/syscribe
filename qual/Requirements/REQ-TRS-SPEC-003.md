---
id: REQ-TRS-SPEC-003
type: Requirement
name: The validation-code catalogue shall explain every emitted finding code, and explain_finding shall return that explanation rather than a severity word
status: draft
reqDomain: software
verificationMethod: test
---

The embedded validation-code catalogue (`prompts/spec/validation.md`, surfaced by `syscribe spec validation` and by the MCP `explain_finding` tool) **shall** contain a row with a non-empty, non-severity explanation for **every** finding code (`E*`, `W*`, `I*`) the implementation can emit. Each row's explanation **shall** match the code's actual meaning in the implementation — in particular `W007` is "a definition is never used as a supertype or type" (unrecognised frontmatter keys are `W047`), and `W010` is test-result ingestion (the product-line unbound-required-parameter warning is `W017`). A code that is documented but never emitted (e.g. `E003`, `E024`, `W301`) **shall** be marked retired rather than described as active.

The MCP `explain_finding` tool **shall** return the explanation column of the matching catalogue row, identified by the table's header (`Condition`/`Description`), regardless of whether the table carries an extra `Severity` column; it **shall never** return a bare severity word (`error`, `warning`, `info`) as the explanation.

**Source:** GH #128 — `explain_code` read the second cell of the first matching row, which for three-column `| Code | Severity | Condition |` tables is the severity; about 35 emitted codes were missing from the catalogue; the `W007`/`W010` rows described stale meanings.

**Acceptance criteria:** `explain_finding` for `E600`, `W610`, `W041`, `E317` returns a sentence, not `error`/`warning`; `explain_finding` for `E108`, `W047`, `W090`, `W563`, `I010` returns an explanation (not "unknown finding code"); `explain_finding W007` describes supertype/type usage, not unrecognised keys; `explain_finding W010` describes test results, not parameters; a test-time source scan of every emitted code finds a non-severity explanation for each in the catalogue.
