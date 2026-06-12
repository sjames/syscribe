---
id: REQ-TRS-LIB-003
type: Requirement
name: Tool shall check dimensional consistency between an element's quantity type and its unit
status: draft
reqDomain: software
verificationMethod: test
---

When an element (or an inline feature/attribute) declares **both** a recognised quantity
type (`typedBy:` a recognised `ISQ` value type, [[REQ-TRS-LIB-002]]) **and** a recognised
unit (`unit:` a recognised `SI` unit or symbol), the tool **shall** verify that the unit's
**physical dimension** matches the quantity type's dimension, and flag a mismatch — so
`typedBy: ISQ::MassValue` with `unit: SI::metre` is caught, while `unit: SI::kilogram` is
clean.

### Dimension model

- Every recognised `SI` unit and `ISQ` quantity-value type maps to a **dimension** over
  the **seven SI base quantities** — Length (L), Mass (M), Time (T), Electric Current (I),
  Thermodynamic Temperature (Θ), Amount of Substance (N), Luminous Intensity (J) — as an
  integer exponent vector. Examples:
  - `MassValue` / `kilogram` / `gram` → `[M¹]`
  - `LengthValue` / `metre` / `kilometre` → `[L¹]`
  - `ForceValue` / `newton` → `[L¹ M¹ T⁻²]`
  - `EnergyValue` / `joule` → `[L² M¹ T⁻²]`
  - `PowerValue` / `watt` → `[L² M¹ T⁻³]`
  - `ElectricPotentialValue` / `volt` → `[L² M¹ T⁻³ I⁻¹]`
  - `DimensionlessValue` / ratio → `[]` (all zero)
- The dimension is **prefix- and magnitude-independent**: `kg`, `g`, `mg`, `tonne` all
  carry dimension `[M¹]`; `km`, `m`, `mm` all carry `[L¹]`. Only the dimension is
  compared, never the magnitude.

### Validation

- For each element/feature carrying **both** a recognised quantity type and a recognised
  unit, the tool **shall** compute both dimensions; if they differ it **shall** raise
  warning **`W044`**, naming the quantity type, the unit, and both dimensions.
- The check is **lenient**: if **either** side is unrecognised (a non-`ISQ` `typedBy`, or
  a non-SI/domain `unit:` such as `USD`/`kWh`/`percent`, or a unit the curated registry
  does not list) the tool **shall not** raise `W044` — it cannot verify what it cannot
  resolve. (Extending the curated registry extends the coverage.)
- `W044` is a **warning** (advisory, gateable with `--deny W044`).

### Crate evaluation (implementation note)

Existing Rust units crates were evaluated and **not** adopted:
- **`uom`**, **`dimensioned`** — type-/compile-time dimensional safety for numeric Rust
  values; they cannot map an arbitrary **unit string** parsed from YAML to a dimension at
  runtime, and do not use SysMLv2 `ISQ`/`SI` names.
- **`rink-core`** — a runtime units calculator with dimensional analysis, but a heavy
  dependency with its own units database whose names do not match `ISQ::MassValue` /
  `SI::kilogram`.

The chosen approach is a **small in-tree static dimension table** (an `i8` exponent vector
over the seven base quantities) keyed by the curated `SI`/`ISQ` registry of
[[REQ-TRS-LIB-002]]. It is deterministic, pure-Rust, dependency-free, and matches SysMLv2
naming exactly — consistent with the project's preference for vendored/pinned
deterministic engines (`ADR-FM-002`). (A future move to a full units engine such as
`rink-core` would warrant its own ADR.)

**Acceptance criteria:**

- `typedBy: ISQ::MassValue` + `unit: SI::kilogram` (and `unit: SI::kg`, `unit: kg`)
  validates with **no** `W044`.
- `typedBy: ISQ::MassValue` + `unit: SI::metre` raises `W044` naming both dimensions
  (mass vs length).
- `typedBy: ISQ::ForceValue` + `unit: SI::newton` is clean; `+ unit: SI::watt` raises
  `W044`.
- `typedBy: ISQ::MassValue` + `unit: USD` raises **no** `W044` (unit unrecognised →
  lenient); a recognised quantity type with no `unit:` raises nothing.
