---
id: REQ-TRS-VAL-018
type: Requirement
name: Tool shall enforce the documented ConfirmationMeasure status set and the Zone/Conduit status set and Security Level range
status: draft
reqDomain: software
verificationMethod: test
---

The format documents closed value sets for three fields that the tool previously parsed but
never checked, so an out-of-contract value (for example `achievedSL: 7`) validated clean. The
tool **shall** enforce each documented constraint:

| Code | Condition |
|---|---|
| `E924` | A `ConfirmationMeasure`'s `status:` is not one of `planned` · `in_progress` · `completed` (§8.18.2, [[REQ-TRS-SAFE-007]]). |
| `E925` | A `targetSL:` or `achievedSL:` on a `Zone`, `Conduit`, `PartDef` or `Part` is outside the IEC 62443-3-3 Security Level range `1`–`4` (§13.2–§13.4, [[REQ-TRS-TYPE-020]]). |
| `E926` | A `Zone`'s or `Conduit`'s `status:` is not one of `draft` · `review` · `approved` · `deprecated` (§13.2, §13.3). |

- These are errors, consistent with every other lifecycle-status enumeration the tool checks
  (`E007` Requirement/TestCase, `E304` ADR, `E702` ReviewRecord, `E708` PlanningItem).
- A missing `status:`/`targetSL:` remains the existing presence check (`E847`/`E950`/`E952`);
  `E924`/`E926` fire only on a present, out-of-set value, and `E925` only on a present,
  out-of-range value. The SL gap warnings (`W950`/`W951`) are unchanged.
- The message names the offending value and the allowed set/range.

**Source:** GH #136 (v0.40.1 documentation/spec consistency review); spec §8.18.2, §13.

**Acceptance criteria:**

- A `ConfirmationMeasure` with `status: approved` raises `E924`; `status: completed` does not.
- A `Zone` with `achievedSL: 7` raises `E925`; a `Conduit` with `achievedSL: 0` raises `E925`;
  a `PartDef` with `targetSL: 5` raises `E925`; values 1–4 raise none.
- A `Zone` with `status: active` and a `Conduit` with `status: done` each raise `E926`;
  `approved` raises none.
- The shipped models raise none of `E924`–`E926`.
