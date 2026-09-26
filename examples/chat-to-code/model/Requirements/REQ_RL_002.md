---
type: Requirement
id: REQ-RL-002
name: The library shall let callers either be rejected or wait
reqClass: stakeholder
reqDomain: system
status: verified
verificationMethod: test
---

When a call is not currently allowed, the caller shall be able to choose between being rejected with an indication of when to retry, and blocking until the call is allowed.

## Rationale

Some callers (request handlers) prefer to fail fast and report a retry time; others (background workers) prefer to wait.

