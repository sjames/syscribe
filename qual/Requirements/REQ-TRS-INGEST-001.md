---
id: REQ-TRS-INGEST-001
type: Requirement
name: Tool shall ingest a session-log report of manual/exploratory verification
status: draft
reqDomain: software
verificationMethod: test
---

`syscribe ingest-results --format session-log <file>` **shall** parse `<file>` as a JSON array
of records, each shaped:

```json
{"testCase": "TC-X-001", "scenario": "...", "steps": [...], "result": "pass|fail|unknown",
 "timestamp": "..."}
```

and persist a verdict per (`testCase`, `scenario`) pair to the results sidecar
(`<model_root>/.syscribe/results.json`), alongside the existing per-function verdicts written by
`cargo-json`/`junit` ingestion.

Unlike `cargo-json`/`junit`'s tolerant, line-skipping parsing, this format **shall** be strict: a
record with an empty or missing `testCase`, `scenario`, or `steps` (or a `steps` that is present
but empty), or an unrecognized `result` value, **shall** fail the whole ingestion with a clear
error naming the offending record — nothing is written, and any existing sidecar is left
untouched. An empty input array **shall** likewise fail ingestion rather than silently writing
an empty result set.

`--format session-log` **shall** never be inferred from the file extension (unlike `cargo-json`/
`junit`) — it must always be named explicitly.

**Motivation:** verifying a feature that requires a live end-to-end session (server restarts,
requests issued as different roles, a throwaway probe client, reading back stored rows) produces
evidence that fits no `#[test]` function `cargo test` would run standalone. Without a
machine-checkable ingestion path, the only record of that verification is a hand-written "Verified
live: ..." sentence — cheap for an LLM (or a human under time pressure) to write whether or not
it is true, and indistinguishable from an unverified claim to `audit`/`validate`.

**Source:** GitHub issue #113.

**Acceptance criteria:**
- A well-formed `session-log` file parses into `.syscribe/results.json` alongside any existing
  `cargo-json`/`junit` data.
- A record with empty/missing `steps` fails ingestion with an error naming the record; nothing is
  written, and the existing sidecar (if any) is unchanged.
- A record with an unrecognized `result` value fails ingestion the same way.
- An empty input array fails ingestion the same way.
