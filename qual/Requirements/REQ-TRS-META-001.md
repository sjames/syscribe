---
id: REQ-TRS-META-001
type: Requirement
title: Tool shall support stereotypes as MetadataDef applications on elements (SysMLv2 metadata, not UML)
status: draft
reqDomain: software
verificationMethod: test
---

SysMLv2 has **no** UML-style stereotype/profile mechanism; the equivalent is a **`metadata
def`** applied to an element via a metadata annotation (with `SemanticMetadata` giving the
classifier-like flavour). The tool **shall** model a "stereotype" exactly this way — a
**`MetadataDef`** applied to elements through their **`metadata:`** field — with validation,
tagged values, rendering, and querying. (This activates the existing `metadata:` frontmatter
field and the recognised `MetadataDef`/`Metadata` types, [[REQ-TRS-TYPE-013]].)

### Defining a stereotype

- A **`MetadataDef`** element defines a stereotype. Its **`features:`** are the stereotype's
  attributes (the old "tagged values").
- A `MetadataDef` **may** declare an optional **`annotates:`** list of metaclass names
  (e.g. `[PartDef, Part]`, or the abstract `Element`/`Definition`/`Usage`) constraining which
  element kinds it may annotate — the analog of UML "extends metaclass". When absent, it may
  annotate any element.

### Applying a stereotype

- Any element **may** apply one or more stereotypes via its **`metadata:`** list. Each entry
  is **either**:
  - a **bare reference** to a `MetadataDef` by qualified name or id —
    `metadata: [Safety::Critical]`; **or**
  - a **map carrying tagged values** — `metadata: [{type: Safety::Critical, level: 3}]` —
    where **`type`** (aliases `apply`/`def`) names the `MetadataDef` and the remaining keys
    set its attribute values.

### Validation

- A `metadata:` entry whose reference does **not** resolve to a `MetadataDef` **shall** raise
  error **`E317`** (unresolved stereotype application; the root-name hint of
  [[REQ-TRS-XREF-006]] applies).
- Applying a `MetadataDef` whose **`annotates:`** does not include the annotated element's
  type (directly or via the abstract `Element`/`Definition`/`Usage` metaclasses) **shall**
  raise error **`E318`** (stereotype not applicable to this element kind). Standard-library
  metadata packages (`ModelingMetadata`, `RiskMetadata`) are recognised and do not raise
  `E317`.
- A tagged-value key in an application that is **not** declared by the `MetadataDef`'s
  `features:` **shall** raise warning **`W045`** (advisory — likely typo / undeclared
  attribute).

### Surfacing

- **`show`** **shall** display an element's applied stereotypes as **«Name»** (with their
  tagged values) in the detail view.
- **`list <Type> --metadata <Def>`** **shall** keep only elements that apply that
  `MetadataDef` (resolved by qualified name or id).

(Rendering applied stereotypes as **«Name» banners in diagrams** is the companion
[[REQ-TRS-META-002]].)

**Source:** GH discussion — SysMLv2 has no UML stereotypes; model them with `metadata def`
applications (option A). Reuses the `MetadataDef`/`Metadata` types and the `metadata:` field.

**Acceptance criteria:**

- A `MetadataDef` `Safety::Critical` with a `level` feature, applied as
  `metadata: [Safety::Critical]` (or `[{type: Safety::Critical, level: 3}]`) on a `PartDef`,
  validates clean.
- `metadata: [Safety::Nope]` (unresolved) raises `E317`; an application of a `MetadataDef`
  whose `annotates: [Requirement]` to a `PartDef` raises `E318`.
- A tagged value `{type: Safety::Critical, lvl: 3}` (key `lvl` not a declared feature) raises
  `W045`; using the declared `level` does not.
- `show` on the annotated element prints `«Critical»` (with `level = 3`); `list PartDef
  --metadata Safety::Critical` lists it.
