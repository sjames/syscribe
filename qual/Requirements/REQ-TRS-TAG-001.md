---
id: REQ-TRS-TAG-001
type: Requirement
name: Generic Tag Filter
title: Tool shall support a generic, orthogonal tag filter across queries and reports
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** recognise a free-text `tags:` list on element frontmatter and provide a `--tag` filter on query and reporting commands (at least `list`, `matrix` row filtering, and `refs`/`links` where applicable). A repeated `--tag` **shall** combine with OR semantics.

Tags are **orthogonal classification labels**, not part of the variability model. The tool **shall**:

- treat `tags:` entries as free text — **not** resolved cross-references — and apply no referential-integrity check (a misspelt tag is not an error);
- never let tags participate in variant projection or in `matrix` column derivation, which remain governed exclusively by `appliesWhen:` and `Configuration` elements (see [[REQ-TRS-VAR-004]]).

**Source:** Issue #9 discussion — decision: tags **complement** `appliesWhen:`, they do not replace it.

**Acceptance criteria:** An element carrying `tags: [x]` is selected by `--tag x` and excluded otherwise; `--tag` filters `matrix` rows without changing the columns; a misspelt tag produces no error (free-text by design); tags have no effect on N/A / coverage classification.
