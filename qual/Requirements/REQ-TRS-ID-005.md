---
id: REQ-TRS-ID-005
type: Requirement
name: Configurable Stable-ID Suffix Width
title: Tool shall accept 3-8 digit stable-ID suffixes with a configurable maximum
status: draft
reqDomain: software
verificationMethod: test
---

Every stable identifier (REQ-*, TC-*, TP-*, ADR-*, CONF-* and the safety/security
ids) ends in a numeric suffix. The tool **shall** accept a suffix of **3 to 8 digits
by default** — widening the previous fixed 3-digit suffix — and **shall** make the
**maximum configurable**.

### Suffix width

- The structural id grammar **shall** recognise a trailing numeric group of **3 or
  more** digits (`^PREFIX(-[A-Z0-9]{2,12})+-[0-9]{3,}$`). A suffix of fewer than 3
  digits **shall** still fail with `E006` (the minimum is fixed at 3). Recognition is
  config-independent so a reference to a long id always resolves.
- The **maximum** number of suffix digits is `id_max_digits`, **default 8**. A stable
  id whose trailing digit group is **longer** than `id_max_digits` **shall** produce
  error **`E023`** naming the id, its digit count, and the configured maximum.
- Existing 3-digit ids remain valid; a 3-to-`id_max_digits`-digit suffix is accepted
  with no finding.

### Configuration

- `id_max_digits` **shall** be read from `[ids] max_digits` in
  `<model_root>/.syscribe.toml`; when absent it **shall** default to **8**.
- A configured value below the fixed minimum of 3 **shall** be clamped to 3 (a max
  cannot be lower than the min).
- Setting `max_digits` higher than 8 (e.g. 12) **shall** raise the cap accordingly;
  setting it lower (e.g. 4) **shall** tighten it.

### Unchanged

- `next-id` continues to zero-pad to a minimum of 3 digits and grows past `999`
  naturally; it is not gated on `id_max_digits`.

**Source:** GH #41; refines the ID scheme of [[REQ-TRS-ID-001]] / [[REQ-TRS-ID-002]] /
[[REQ-TRS-ID-003]]; CLAUDE.md §ID Scheme; §11.12.

**Acceptance criteria:**

- With no `[ids]` config: `REQ-X-001` (3) and `REQ-X-00000001` (8) validate clean;
  `REQ-X-000000001` (9) raises `E023`; `REQ-X-01` (2) raises `E006`.
- With `.syscribe.toml [ids] max_digits = 9`: `REQ-X-000000001` (9) validates clean.
- With `.syscribe.toml [ids] max_digits = 4`: `REQ-X-00001` (5) raises `E023`.
- A reference to an over-long id still resolves (no `E102`); the defect surfaces as
  `E023` on the id-bearing element, not as a dangling reference.
