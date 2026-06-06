---
id: REQ-TRS-OUT-007
type: Requirement
name: Structured Model Graph Export
title: Tool shall emit a versioned, machine-readable export of the whole model graph
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide an `export` command that emits the entire model graph in a machine-readable form, so external tooling can consume the model without re-implementing the parser. The export **shall**:

- emit one record per element carrying `qname`, `file`, `id` (when present), `type`, `name` (when present), and the element's typed frontmatter fields;
- include, for each requirement, the resolved reverse-index relationships under `computed` — `verifiedBy` (covering TestCase ids) and `derivedChildren` (child requirement ids);
- preserve the fields needed to reconstruct traceability — TestCase `sourceFile`, `testLevel`, `status`, `testFunctions`, and Requirement `verifies`/`derivedFrom`/`satisfies` — without reading the Markdown body;
- carry a top-level `schemaVersion` so consumers can detect breaking changes;
- support a default pretty-JSON document and an `--ndjson` (newline-delimited) variant.

**Source:** Issue #2 (structured model graph export)

**Acceptance criteria:** `export` emits valid JSON with `schemaVersion` and an `elements` array; a requirement element exposes `computed.verifiedBy` listing its covering TestCases; a TestCase element exposes its `verifies` list — both round-tripped from frontmatter without parsing the body.
