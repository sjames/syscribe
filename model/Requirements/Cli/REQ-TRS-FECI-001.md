---
type: Requirement
id: REQ-TRS-FECI-001
name: "A product-line fixture and contract tests let a front end's CI run against the real binary"
status: approved
reqDomain: software
reqClass: system
tags:
  - lsp
  - ple
---

The repository shall provide what a front end's CI needs to test against the real `syscribe` binary (GH #430):

- a fixture model, `examples/ple-fixture/`, that validates with no errors and covers alternative and or groups, cross-tree constraints, typed parameters with window, range, enumeration and unit, parameter constraints, a staged `derivedFrom:` chain of Configurations plus an unstaged one, requirements with tests on two rigs and a planning tree, with `EXPECTED.json` stating its contents and the answers of documented calls;
- contract tests that run the binary against the fixture over every transport — every published CLI JSON contract validates against its schema, every advertised LSP request answers with a schema-valid payload and a saved change produces `syscribe/modelChanged`, the MCP server offers every advertised read tool and hides write tools when read-only — and that check the required capabilities; a contract or request added without a case fails the suite;
- a guide, `docs/guides/frontend-ci.md`, saying how a client gates on `capabilities`, asserts the schemas and tests version skew, in no particular language.
