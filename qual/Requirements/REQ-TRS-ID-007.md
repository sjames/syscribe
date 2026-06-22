---
id: REQ-TRS-ID-007
type: Requirement
name: Tool shall accept configurable additional stable-ID prefixes per element type
status: draft
reqDomain: software
verificationMethod: test
---

Each id-identified element type has a fixed built-in stable-ID prefix (`REQ` for
`Requirement`, `TC` for `TestCase`, `ADR` for `ADR`, … `FEAT` for `FeatureDef`). A
project **shall** be able to declare **additional** prefixes that are accepted as valid
stable IDs for a given type, so that — for example — both `REQ-SCHED-001` and
`STK-SCHED-001` are valid `Requirement` ids in the same model.

The mechanism is **general** (it applies to every id-identified type), **additive** (a
configured prefix is accepted *in addition to* the built-in, which always stays valid),
and **pure identity** (a prefix affects only ID validation and id-based resolution; it
carries no other semantics and is independent of `reqClass`, `domain`, and every other
field).

### Configuration

- Additional prefixes **shall** be read from `[ids.prefixes]` in
  `<model_root>/.syscribe.toml`, a table keyed by **element type name** (the value of
  `type:`) whose value is a list of prefix strings:

  ```toml
  [ids.prefixes]
  Requirement = ["STK", "SYS"]
  TestCase    = ["QT"]
  ```

- The table is **opt-in**: when `[ids.prefixes]` is absent every type accepts only its
  built-in prefix, exactly as before (no behaviour change).
- An additional prefix **shall** follow the same shape as the built-in prefixes:
  `^[A-Z][A-Z0-9]{1,11}$` — uppercase, starting with a letter, 2–12 characters. The
  built-in prefix of the type need not be repeated; it is always accepted.

### Accepted id grammar

- A stable id under an additional prefix `P` **shall** be recognised by the same grammar
  as the type's built-in ids, with `P` substituted for the built-in prefix:
  - for the suffix-bearing types: `^P(-[A-Z0-9]{2,12})+-[0-9]{3,}$`;
  - for `FeatureDef` (which has no mandatory numeric suffix, [[REQ-TRS-ID-006]]):
    `^P(-[A-Z0-9]{2,12})+$`.
- The configured-maximum suffix-digit cap ([[REQ-TRS-ID-005]], `E023`) **shall** apply
  identically to ids under an additional prefix.
- An element whose `id` matches its type's built-in **or** any configured additional
  prefix **shall not** raise `E006`. An `id` that matches neither **shall** still raise
  `E006`.
- An id under a configured prefix **shall** be indexed and resolvable by that id exactly
  like a built-in-prefixed id (`verifies:`, `derivedFrom:`, `satisfies:`, the qname
  basic-name exemption of `W042`, and `is_stable_id`-gated behaviour all apply).

### Malformed configuration

- An `[ids.prefixes]` entry keyed by a name that is **not** a known id-identified element
  type **shall** raise warning **`W046`** naming the unknown key; the entry is ignored.
- A prefix string that does **not** match `^[A-Z][A-Z0-9]{1,11}$` **shall** raise
  warning **`W046`** naming the offending prefix and its type; that prefix is ignored
  while well-formed siblings in the same list still take effect.

### Unchanged

- Built-in prefixes are always valid; removing or replacing them is **not** supported
  (the list is strictly additive).
- `next-id` continues to operate on whatever prefix string it is given; it is not gated
  on `[ids.prefixes]`.

**Source:** GH (user request — "support different prefixes for requirements, e.g. REQ and
STK"); extends the ID scheme of [[REQ-TRS-ID-005]] / [[REQ-TRS-ID-006]]; CLAUDE.md
§ID Scheme; §11.12.

**Acceptance criteria:**

- With no `[ids.prefixes]` config: `STK-SCHED-001` on a `Requirement` raises `E006`
  (unchanged baseline).
- With `[ids.prefixes] Requirement = ["STK"]`: a `Requirement` with `id: STK-SCHED-001`
  validates clean, and a `TestCase` whose `verifies:` lists `STK-SCHED-001` resolves
  (no `E102`); `REQ-SCHED-001` on a `Requirement` still validates clean (additive).
- A `Requirement` with `id: STK-SCHED-000000001` (9-digit suffix) under the same config
  raises `E023` (the digit cap applies to additional prefixes too).
- `STK-SCHED-001` configured for `Requirement` is **not** accepted as a `TestCase` id
  (per-type; a `TestCase` carrying it raises `E006`).
- `[ids.prefixes] Frobnicate = ["XX"]` raises `W046` (unknown type); a prefix `"st-k"`
  or `"1AB"` raises `W046` (malformed) and is ignored.
