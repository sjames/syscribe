---
id: REQ-TRS-SYSMLV2-024
type: Requirement
name: "A SysMLv2 flow def/flow maps to the native FlowDef/Flow schema; a nested flow usage's endpoints also lift onto the owning part's flowConnections:"
status: verified
reqDomain: software
verificationMethod: test
---

A `flow def` shall be synthesized into a native `FlowDef` element carrying `supertype:`/`doc`. A
*named* `flow`/`message`/`succession flow` usage shall be synthesized into a native `Flow` element
carrying `itemType:`/`doc`. Additionally, every `FlowUsage` (named or anonymous) found directly
inside a `part def`/`part` usage body shall lift its `from:`/`to:`/`kind:`/`item:` onto the *owning*
part's own `flowConnections:` field — the exact dual pattern `REQ-TRS-SYSMLV2-010` already
established for `connect` statements and `connections:`.

**Source:** `REQ-TRS-SYSMLV2-024` (product model), `ADR-SYS-SYSMLV2-001` addendum.
