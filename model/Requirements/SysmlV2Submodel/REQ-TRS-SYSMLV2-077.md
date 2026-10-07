---
type: Requirement
id: REQ-TRS-SYSMLV2-077
name: "accept and send carry their via and to targets natively"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-SYSMLV2-000]
breakdownAdr: Decisions::SysmlV2SubmodelADR
tags:
  - sysmlv2
---

`accept x [: T] via p;` and `send x [: T] [via p] [to t];` in an action body shall be ingested as `AcceptAction`/`SendAction` `subActions:` entries with `via: p` (and, for send, a new native optional `to: t` sub-field, documented in spec 8.7.6). `export-sysml` shall write `via`/`to` as that native syntax and no longer as an `@SyscribeStep` annotation. An existing `@SyscribeStep { via = '...'; }` annotation is still ingested (deprecated, documented).
