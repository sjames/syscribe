---
id: REQ-TRS-VAR-002
type: Requirement
title: Tool shall derive TestCase-to-Configuration membership from appliesWhen
status: draft
reqDomain: software
verificationMethod: test
---

A `TestCase`'s variant membership **shall** be expressed with the same `appliesWhen:` mechanism as every other element — there is **no** dedicated `runsIn` field.

The tool **shall** treat a `TestCase` as *running in* a `Configuration` C **iff** the `TestCase`'s `appliesWhen:` condition is satisfied by C's feature selections. A `TestCase` with no `appliesWhen:` is *configuration-agnostic* and runs in every `Configuration`.

The tool **shall** surface this relationship in both directions:

- `links <TC>` shows the `TestCase`'s outbound `appliesWhen:` condition;
- `refs <CONF>` lists the `TestCase`s whose `appliesWhen:` the `Configuration` satisfies (computed by evaluation, not stored as an edge).

**Source:** Issue #8

**Acceptance criteria:** `links <TC>` shows the outbound `appliesWhen:` condition; `refs <CONF>` lists the inbound `TestCase`s computed by evaluating each `TestCase`'s `appliesWhen:` against the `Configuration`; a `TestCase` with no `appliesWhen:` appears under every `Configuration`; an unresolved `appliesWhen:` operand is `E209`.
