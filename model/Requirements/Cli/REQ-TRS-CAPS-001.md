---
type: Requirement
id: REQ-TRS-CAPS-001
name: "capabilities tells a front end what the installed binary supports"
status: approved
reqDomain: software
reqClass: system
tags:
  - cli
  - ple
---

`syscribe capabilities --json` shall return, without a model, the tool version, every command with its checked options, the published JSON contracts with their versions, which optional capabilities exist, the MCP tool names and the custom LSP requests (GH #414). The LSP `initialize` result shall carry the same document under `capabilities.experimental.syscribe`.

An optional capability is `true` only when every command (or command option) it needs exists, so a front end tests what it wants rather than comparing version numbers.
