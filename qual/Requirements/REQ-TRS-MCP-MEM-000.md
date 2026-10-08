---
id: REQ-TRS-MCP-MEM-000
type: Requirement
name: The MCP server keeps its memory use bounded and observable on models of at least 12,000 elements
status: verified
reqDomain: software
verificationMethod: test
---

The MCP server **shall** serve a model of at least 12,000 elements within a documented memory
budget through load and guarded writes, **shall not** grow its resident memory with each write, and
**shall** report its own resident and peak resident memory, element count and per-element record
size through a `server_stats` tool.

**Source:** `REQ-TRS-MCP-MEM-000` (product model).

**Acceptance criteria:** (a) `server_stats` reports element count, body bytes, element record size
and, on Linux, resident and peak resident memory; (b) an element record is under 3,000 bytes;
(c) a 12,000-element model stays under 160 MB resident after load; (d) three guarded writes keep
the peak under 300 MB; (e) resident memory after the later writes is within 25 MB of that after
the first.
