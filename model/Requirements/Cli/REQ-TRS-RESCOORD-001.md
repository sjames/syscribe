---
type: Requirement
id: REQ-TRS-RESCOORD-001
name: "A result record names the condition it exercised and the unit it ran on, and both are part of its key"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - results
---

A result record shall accept `condition`, a map of axis to point name (`{"ambient": "hot"}`), and `provenance.unit`, `{id, of?, lot?}` (ADR-SYS-COVERAGE-001, GH #500, GH #501). The record's key shall be its test case, scenario, environment, condition and unit, so that a later ingest supersedes only the record with the same key and a result at another condition, or on another unit, is kept beside it rather than replacing it. A condition axis shall be a basic name and its point non-empty, a unit shall have a non-empty `id`, and a malformed one shall reject the whole file with nothing written. `unit.of` shall resolve to a part definition and be stored by qualified name; one that resolves to nothing, or to another kind of element, shall reject the file. `ingest-results` shall accept `--unit`, `--unit-of`, `--lot` and repeatable `--condition axis=point` for the records-like formats (`records`, `robot`, `cucumber`, `plugin:`), filling only what a record does not name itself and holding the filled values to the same rules. A record with neither field shall behave, be stored and be keyed exactly as before. A failing record at any condition or on any unit shall still fail the TestCase's verdict; judging a plan against its required conditions and units is a later slice.
