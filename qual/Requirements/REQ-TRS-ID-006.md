---
id: REQ-TRS-ID-006
type: Requirement
name: Tool shall require every FeatureDef to carry a stable id and allow features to be referenced by it
status: draft
reqDomain: software
verificationMethod: test
---

A `FeatureDef` is name-identified (Style B): its identity segment is its basic name
(e.g. `Features::Anti_Lock`). So that every feature can be referenced by a stable id
independent of its path/name, a `FeatureDef` **shall** additionally carry a stable
**`id`** (a shortName), mirroring the stable-id types. The id is a *mandatory* second
axis; the feature's **name** remains its identity segment and label.

### Stable id on a FeatureDef

- Every `FeatureDef` **shall** declare an `id` matching the prefix pattern
  `^FEAT(-[A-Z0-9]{2,12})+$` — prefix `FEAT` followed by one or more
  uppercase-alphanumeric segments (2–12 chars each). Unlike the other stable-id types, a
  feature id **need not** end in a numeric segment: both `FEAT-ABS` and `FEAT-ABS-001`
  are valid. A `FeatureDef` with **no** `id` **shall** raise error **`E201`** (the PLE
  required-field error, shared with `Configuration`); the `E023` digit-cap applies only
  to a *numeric* trailing segment, so it never fires on a name-only feature id.
- The `id` is a **mandatory secondary axis**: the FeatureDef's **name** remains its
  identity segment and label and **shall** still follow the basic-name grammar (`W042`);
  a `title:` on a FeatureDef is `E025` (it is name-labelled). A malformed `FEAT` id
  **shall** raise `E006`; a duplicate `id` **shall** raise the generic `E101`.
- An element with a `FEAT` id **shall** be indexed and resolvable by that id.

### Referencing a feature by id

- A feature reference **shall** resolve to its `FeatureDef` by **either** its qualified
  name (`Features::Anti_Lock`) **or** its stable id (`FEAT-ABS-001`). This applies in:
  - **`appliesWhen:`** boolean expressions, and
  - a **`Configuration`'s `features:`** keys (a selection may key on the id or the qname).
- The `appliesWhen` tokenizer **shall** accept a stable-id-shaped feature token (a `-`
  is unambiguous in `appliesWhen` — it is a boolean grammar with no subtraction
  operator). A reference whose token is **not** a valid stable id and whose qname
  segment is **not** a basic name (e.g. `Features::Anti-Lock`) **shall** still be
  rejected (`E209`) — this preserves the basic-name rule for feature *names*
  ([[REQ-TRS-NAME-001]]); only the stable-`id` form may contain hyphens.
- `parameterConstraints` (arithmetic) continue to reference feature parameters by
  **qualified name only** — `-` is the subtraction operator there, so the id form is
  not accepted in that context.
- An `appliesWhen`/`features:` operand that resolves by neither key **shall** raise
  `E209`.

**Source:** GH discussion (identity-model unification, "keep id primary"); SysMLv2
short-name concept. Refines [[REQ-TRS-NAME-001]], [[REQ-TRS-PROJ-001]].

**Acceptance criteria:**

- A `FeatureDef` with `id: FEAT-ABS-001` and a basic `name` validates clean; a
  `FeatureDef` with **no** `id` raises `E201`; a malformed `FEAT` id raises `E006`; two
  FeatureDefs sharing a `FEAT` id raise `E101`.
- `appliesWhen: FEAT-ABS-001` resolves with **no** `E209` and gates an element
  identically to `appliesWhen: Features::Anti_Lock` (same active/inactive result across
  configurations).
- A `Configuration` selecting the feature via `features: { FEAT-ABS-001: true }`
  projects identically to selecting it via the qname key.
- `appliesWhen: Features::Anti-Lock` (a hyphenated *name* reference, not a stable id)
  still raises `E209` — the hyphen relaxation applies only to the stable-id form.
- An `appliesWhen` operand that matches neither a FeatureDef qname nor a `FEAT` id
  raises `E209`.
