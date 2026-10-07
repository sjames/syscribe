---
id: REQ-TRS-SYSMLV2-077
type: Requirement
name: "accept and send carry their via and to targets natively"
status: verified
reqDomain: software
verificationMethod: test
---

`accept x [: T] via p;` and `send x [: T] [via p] [to t];` in an action body shall be ingested as `AcceptAction`/`SendAction` `subActions:` entries with `via: p` (and, for send, a new native optional `to: t` sub-field, documented in spec 8.7.6). `export-sysml` shall write `via`/`to` as that native syntax and no longer as an `@SyscribeStep` annotation. An existing `@SyscribeStep { via = '...'; }` annotation is still ingested (deprecated, documented).

**Source:** `REQ-TRS-SYSMLV2-077` (product model), `ADR-SYS-SYSMLV2-001`.
