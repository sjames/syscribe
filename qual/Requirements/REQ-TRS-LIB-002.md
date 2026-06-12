---
id: REQ-TRS-LIB-002
type: Requirement
name: Tool shall recognise SI units and ISQ quantity-value types as a lenient curated library
status: draft
reqDomain: software
verificationMethod: test
---

Beyond the closed auto-imported packages of [[REQ-TRS-LIB-001]] (`ScalarValues`, `Base`),
the tool **shall** recognise a curated set of the **`ISQ`** quantity-value types (for
`typedBy:`) and **`SI`** units (for `unit:` and type references) so that common physical
quantities resolve cleanly and are available to the dimensional-consistency check
([[REQ-TRS-LIB-003]]). Because `ISQ`/`SI` are **large and open** packages — and the
`unit:` field legitimately carries non-SI domain units (`USD`, `kWh`, `percent`, `bits`) —
this is a **lenient** tier: recognised members resolve cleanly, but an *unrecognised*
member of `ISQ`/`SI` is **not** flagged (no `W043`), unlike the closed-package tier.

### Two recognition tiers

| Tier | Packages | Recognised member | Unknown member |
|---|---|---|---|
| **Closed** ([[REQ-TRS-LIB-001]]) | `ScalarValues`, `Base` | resolves clean | **flagged `W043`** |
| **Open / curated** (this requirement) | `ISQ`, `SI` (and `SIPrefixes`) | resolves clean | **lenient** (no `W043`) |

### Recognised members (curated)

- **`ISQ` quantity-value types** — at least: `LengthValue`, `MassValue`, `DurationValue`,
  `AngleValue`, `AreaValue`, `VolumeValue`, `SpeedValue`, `AccelerationValue`,
  `ForceValue`, `PressureValue`, `EnergyValue`, `PowerValue`, `FrequencyValue`,
  `MassFlowRateValue`, `ElectricCurrentValue`, `ElectricPotentialValue` (alias
  `VoltageValue`), `ResistanceValue`, `CapacitanceValue`, `ElectricChargeValue`,
  `ThermodynamicTemperatureValue` (alias `TemperatureValue`), `DimensionlessValue`.
- **`SI` units** — the seven base units (`metre`, `kilogram`, `second`, `ampere`,
  `kelvin`, `mole`, `candela`) and common derived units (`newton`, `pascal`, `joule`,
  `watt`, `volt`, `ohm`, `farad`, `henry`, `coulomb`, `hertz`, `siemens`, `weber`,
  `tesla`, `lumen`, `lux`, `radian`, `degreeCelsius`, `gram`, `litre`, `tonne`).

### Naming and aliases

- Both the **full SysMLv2 name** (`SI::kilogram`, `ISQ::MassValue`) **and** the common
  **short symbol** (`SI::kg`, `SI::m`, `SI::V`, `SI::A`, `SI::W`, `SI::J`) **shall** be
  recognised as the same member (the models use short symbols today).
- A bare unit symbol in `unit:` (e.g. `unit: kg`, `unit: ms`) **may** be recognised
  against the same registry where unambiguous; recognition feeds [[REQ-TRS-LIB-003]] but
  imposes no new error.

### Behaviour

- A `typedBy:`/`returnType:`/`parameters[].type` reference to a recognised `ISQ` value
  type or `SI` unit **shall** resolve cleanly (no `W404`).
- The **`unit:`** field **shall remain permissive**: a recognised `SI` unit is accepted,
  and any other string (a non-SI/domain unit) is **also** accepted — there is **no** new
  error or warning for an unrecognised `unit:` value.
- An unrecognised member of `ISQ`/`SI` (e.g. `ISQ::WibbleValue`, `SI::flurg`) **shall
  not** raise `W043` (lenient tier); only the closed packages flag unknown members.

**Source:** user request to add built-in support for common SI/quantity types. Extends the
standard-library inventory (spec §4.7) and the recognition model of [[REQ-TRS-LIB-001]];
the dimension data underpins [[REQ-TRS-LIB-003]]. The recognition registry is implemented
**in-tree** (a small static table) rather than via an external units crate — see the
crate evaluation note in [[REQ-TRS-LIB-003]].

**Acceptance criteria:**

- `typedBy: ISQ::MassValue`, `typedBy: SI::kilogram` and `typedBy: SI::kg` validate with
  **no** `W404` (including in an operation `returnType`/parameter).
- An unrecognised `ISQ`/`SI` member raises **no** `W043` (lenient), whereas a
  `ScalarValues::Flota` still raises `W043` ([[REQ-TRS-LIB-001]] unaffected).
- `unit: USD`, `unit: kWh`, `unit: percent` validate with no new finding (permissive).
- `SI::kilogram` and `SI::kg` resolve to the same recognised unit (alias).
