---
id: REQ-TRS-BT-025
type: Requirement
name: "Export fails on an unresolved required parameter unless --allow-open is given"
status: draft
reqDomain: software
verificationMethod: test
---

`params export` shall fail naming each required parameter that has no value, default or fallback; with `--allow-open` it shall emit an explicit, marked placeholder for each instead.

**Acceptance criteria:**
- Without the flag, the command exits non-zero and names every unresolved parameter.
- With the flag, the file contains a clearly marked placeholder per unresolved parameter.
