//! Built-in SI units and ISQ quantity-value types with their physical dimensions
//! (REQ-TRS-LIB-002 / REQ-TRS-LIB-003).
//!
//! An **in-tree** static registry over the seven SI base quantities — chosen over
//! external units crates (`uom`/`dimensioned` are compile-time/type-level and cannot map
//! an arbitrary string to a dimension at runtime; `rink-core` is heavy and uses
//! non-SysML names). This table is deterministic, dependency-free, and matches SysMLv2
//! `ISQ`/`SI` naming. See `REQ-TRS-LIB-003` for the rationale.

/// Exponent vector over the seven SI base quantities, in order:
/// Length (L), Mass (M), Time (T), Electric Current (I), Temperature (Θ),
/// Amount of Substance (N), Luminous Intensity (J).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Dim(pub [i8; 7]);

const fn d(l: i8, m: i8, t: i8, i: i8, th: i8, n: i8, j: i8) -> Dim {
    Dim([l, m, t, i, th, n, j])
}

impl Dim {
    /// A compact human label, e.g. `M`, `L·M·T^-2`, or `dimensionless`.
    pub fn human(&self) -> String {
        const SYM: [&str; 7] = ["L", "M", "T", "I", "Θ", "N", "J"];
        let parts: Vec<String> = self
            .0
            .iter()
            .enumerate()
            .filter(|(_, &e)| e != 0)
            .map(|(i, &e)| {
                if e == 1 {
                    SYM[i].to_string()
                } else {
                    format!("{}^{}", SYM[i], e)
                }
            })
            .collect();
        if parts.is_empty() {
            "dimensionless".to_string()
        } else {
            parts.join("·")
        }
    }
}

/// ISQ quantity-value type name (without the `ISQ::` prefix) → dimension.
fn quantity_table(name: &str) -> Option<Dim> {
    Some(match name {
        "LengthValue" => d(1, 0, 0, 0, 0, 0, 0),
        "MassValue" => d(0, 1, 0, 0, 0, 0, 0),
        "DurationValue" | "TimeValue" => d(0, 0, 1, 0, 0, 0, 0),
        "ElectricCurrentValue" => d(0, 0, 0, 1, 0, 0, 0),
        "ThermodynamicTemperatureValue" | "TemperatureValue" => d(0, 0, 0, 0, 1, 0, 0),
        "AmountOfSubstanceValue" => d(0, 0, 0, 0, 0, 1, 0),
        "LuminousIntensityValue" => d(0, 0, 0, 0, 0, 0, 1),
        "AngleValue" | "DimensionlessValue" => d(0, 0, 0, 0, 0, 0, 0),
        "AreaValue" => d(2, 0, 0, 0, 0, 0, 0),
        "VolumeValue" => d(3, 0, 0, 0, 0, 0, 0),
        "TorqueValue" | "MomentOfForceValue" => d(2, 1, -2, 0, 0, 0, 0),
        "AngularVelocityValue" | "AngularSpeedValue" => d(0, 0, -1, 0, 0, 0, 0),
        "AngularAccelerationValue" => d(0, 0, -2, 0, 0, 0, 0),
        "MassDensityValue" | "DensityValue" => d(-3, 1, 0, 0, 0, 0, 0),
        "VolumeFlowRateValue" => d(3, 0, -1, 0, 0, 0, 0),
        "MomentumValue" => d(1, 1, -1, 0, 0, 0, 0),
        "JerkValue" => d(1, 0, -3, 0, 0, 0, 0),
        "SpeedValue" | "VelocityValue" => d(1, 0, -1, 0, 0, 0, 0),
        "AccelerationValue" => d(1, 0, -2, 0, 0, 0, 0),
        "ForceValue" => d(1, 1, -2, 0, 0, 0, 0),
        "PressureValue" => d(-1, 1, -2, 0, 0, 0, 0),
        "EnergyValue" => d(2, 1, -2, 0, 0, 0, 0),
        "PowerValue" => d(2, 1, -3, 0, 0, 0, 0),
        "FrequencyValue" => d(0, 0, -1, 0, 0, 0, 0),
        "MassFlowRateValue" => d(0, 1, -1, 0, 0, 0, 0),
        "ElectricPotentialValue" | "VoltageValue" => d(2, 1, -3, -1, 0, 0, 0),
        "ResistanceValue" => d(2, 1, -3, -2, 0, 0, 0),
        "CapacitanceValue" => d(-2, -1, 4, 2, 0, 0, 0),
        "ElectricChargeValue" => d(0, 0, 1, 1, 0, 0, 0),
        _ => return None,
    })
}

/// SI unit name or common symbol (without the optional `SI::` prefix) → dimension.
/// Dimensions are prefix-independent: `kg`/`g`/`mg`/`tonne` are all mass.
fn unit_table(name: &str) -> Option<Dim> {
    Some(match name {
        "metre" | "meter" | "m" | "kilometre" | "kilometer" | "km" | "millimetre" | "mm"
        | "centimetre" | "cm" => d(1, 0, 0, 0, 0, 0, 0),
        "kilogram" | "kg" | "gram" | "g" | "milligram" | "mg" | "tonne" | "t" => {
            d(0, 1, 0, 0, 0, 0, 0)
        }
        "second" | "s" | "millisecond" | "ms" | "microsecond" | "us" | "minute" | "min"
        | "hour" | "h" => d(0, 0, 1, 0, 0, 0, 0),
        "ampere" | "amp" | "A" | "milliampere" | "mA" => d(0, 0, 0, 1, 0, 0, 0),
        "kelvin" | "K" | "degreeCelsius" | "degC" | "celsius" => d(0, 0, 0, 0, 1, 0, 0),
        "mole" | "mol" => d(0, 0, 0, 0, 0, 1, 0),
        "candela" | "cd" => d(0, 0, 0, 0, 0, 0, 1),
        "newton" | "N" => d(1, 1, -2, 0, 0, 0, 0),
        "pascal" | "Pa" | "kPa" | "MPa" | "bar" => d(-1, 1, -2, 0, 0, 0, 0),
        "joule" | "J" | "kilojoule" | "kJ" | "wattHour" | "Wh" | "kilowattHour" | "kWh" => {
            d(2, 1, -2, 0, 0, 0, 0)
        }
        "watt" | "W" | "kilowatt" | "kW" | "megawatt" | "MW" => d(2, 1, -3, 0, 0, 0, 0),
        "volt" | "V" | "millivolt" | "mV" | "kilovolt" | "kV" => d(2, 1, -3, -1, 0, 0, 0),
        "ohm" => d(2, 1, -3, -2, 0, 0, 0),
        "farad" | "F" => d(-2, -1, 4, 2, 0, 0, 0),
        "henry" | "H" => d(2, 1, -2, -2, 0, 0, 0),
        "coulomb" | "C" => d(0, 0, 1, 1, 0, 0, 0),
        "hertz" | "Hz" | "kilohertz" | "kHz" | "megahertz" | "MHz" => d(0, 0, -1, 0, 0, 0, 0),
        "siemens" => d(-2, -1, 3, 2, 0, 0, 0),
        "weber" | "Wb" => d(2, 1, -2, -1, 0, 0, 0),
        "tesla" => d(0, 1, -2, -1, 0, 0, 0),
        "radian" | "rad" | "steradian" | "sr" | "degree" | "deg" => d(0, 0, 0, 0, 0, 0, 0),
        "rpm" => d(0, 0, -1, 0, 0, 0, 0),
        "newtonMetre" | "newtonMeter" | "Nm" => d(2, 1, -2, 0, 0, 0, 0),
        "litre" | "liter" | "L" | "millilitre" | "mL" => d(3, 0, 0, 0, 0, 0, 0),
        "metrePerSecond" | "mps" | "kilometrePerHour" | "kph" | "kmh" => d(1, 0, -1, 0, 0, 0, 0),
        _ => return None,
    })
}

/// SI prefixes (GH #209): (symbol, long name, decimal exponent). `µ` and `u` both mean micro.
const SI_PREFIXES: &[(&str, &str, i32)] = &[
    ("Y", "yotta", 24), ("Z", "zetta", 21), ("E", "exa", 18), ("P", "peta", 15),
    ("T", "tera", 12), ("G", "giga", 9), ("M", "mega", 6), ("k", "kilo", 3),
    ("h", "hecto", 2), ("da", "deca", 1), ("d", "deci", -1), ("c", "centi", -2),
    ("m", "milli", -3), ("µ", "micro", -6), ("u", "micro", -6), ("n", "nano", -9),
    ("p", "pico", -12), ("f", "femto", -15), ("a", "atto", -18), ("z", "zepto", -21),
    ("y", "yocto", -24),
];

/// Prefixable SI base symbols (never `degC`, `min`, `h`, `rpm`...).
fn prefixable_symbol(name: &str) -> Option<Dim> {
    Some(match name {
        "m" => d(1, 0, 0, 0, 0, 0, 0),
        "g" => d(0, 1, 0, 0, 0, 0, 0),
        "s" => d(0, 0, 1, 0, 0, 0, 0),
        "A" => d(0, 0, 0, 1, 0, 0, 0),
        "K" => d(0, 0, 0, 0, 1, 0, 0),
        "mol" => d(0, 0, 0, 0, 0, 1, 0),
        "cd" | "lm" => d(0, 0, 0, 0, 0, 0, 1),
        "lx" => d(-2, 0, 0, 0, 0, 0, 1),
        "N" => d(1, 1, -2, 0, 0, 0, 0),
        "Pa" => d(-1, 1, -2, 0, 0, 0, 0),
        "J" | "Wh" => d(2, 1, -2, 0, 0, 0, 0),
        "W" => d(2, 1, -3, 0, 0, 0, 0),
        "V" => d(2, 1, -3, -1, 0, 0, 0),
        "Ohm" | "Ω" => d(2, 1, -3, -2, 0, 0, 0),
        "F" => d(-2, -1, 4, 2, 0, 0, 0),
        "H" => d(2, 1, -2, -2, 0, 0, 0),
        "C" => d(0, 0, 1, 1, 0, 0, 0),
        "Hz" | "Bq" => d(0, 0, -1, 0, 0, 0, 0),
        "S" => d(-2, -1, 3, 2, 0, 0, 0),
        "Wb" => d(2, 1, -2, -1, 0, 0, 0),
        "T" => d(0, 1, -2, -1, 0, 0, 0),
        "L" => d(3, 0, 0, 0, 0, 0, 0),
        "Gy" | "Sv" => d(2, 0, -2, 0, 0, 0, 0),
        "kat" => d(0, 0, -1, 0, 0, 1, 0),
        "rad" | "sr" => d(0, 0, 0, 0, 0, 0, 0),
        _ => return None,
    })
}

/// Prefixable SI unit long names.
fn prefixable_long(name: &str) -> Option<Dim> {
    Some(match name {
        "metre" | "meter" => d(1, 0, 0, 0, 0, 0, 0),
        "gram" => d(0, 1, 0, 0, 0, 0, 0),
        "second" => d(0, 0, 1, 0, 0, 0, 0),
        "ampere" => d(0, 0, 0, 1, 0, 0, 0),
        "kelvin" => d(0, 0, 0, 0, 1, 0, 0),
        "mole" => d(0, 0, 0, 0, 0, 1, 0),
        "candela" | "lumen" => d(0, 0, 0, 0, 0, 0, 1),
        "lux" => d(-2, 0, 0, 0, 0, 0, 1),
        "newton" => d(1, 1, -2, 0, 0, 0, 0),
        "pascal" => d(-1, 1, -2, 0, 0, 0, 0),
        "joule" | "wattHour" => d(2, 1, -2, 0, 0, 0, 0),
        "watt" => d(2, 1, -3, 0, 0, 0, 0),
        "volt" => d(2, 1, -3, -1, 0, 0, 0),
        "ohm" => d(2, 1, -3, -2, 0, 0, 0),
        "farad" => d(-2, -1, 4, 2, 0, 0, 0),
        "henry" => d(2, 1, -2, -2, 0, 0, 0),
        "coulomb" => d(0, 0, 1, 1, 0, 0, 0),
        "hertz" | "becquerel" => d(0, 0, -1, 0, 0, 0, 0),
        "siemens" => d(-2, -1, 3, 2, 0, 0, 0),
        "weber" => d(2, 1, -2, -1, 0, 0, 0),
        "tesla" => d(0, 1, -2, -1, 0, 0, 0),
        "litre" | "liter" => d(3, 0, 0, 0, 0, 0, 0),
        "gray" | "sievert" => d(2, 0, -2, 0, 0, 0, 0),
        "katal" => d(0, 0, -1, 0, 0, 1, 0),
        "radian" | "steradian" => d(0, 0, 0, 0, 0, 0, 0),
        _ => return None,
    })
}

/// Split an SI-prefixed unit into `(decimal exponent, dimension)`: `kN` -> `(3, force)`,
/// `MHz` -> `(6, frequency)`, `micrometre` -> `(-6, length)`. An unprefixed table unit yields
/// exponent 0. A symbol prefix applies to a symbol base and a long prefix to a long base
/// (never mixed). Exact table entries win, so `m`, `min`, `mol`, `cd`, `Pa` keep their usual
/// meaning.
pub fn split_si_prefix(name: &str) -> Option<(i32, Dim)> {
    if let Some(dim) = unit_table(name) {
        return Some((0, dim));
    }
    if let Some(dim) = prefixable_symbol(name).or_else(|| prefixable_long(name)) {
        return Some((0, dim));
    }
    for &(sym, long, exp) in SI_PREFIXES {
        if let Some(dim) = name.strip_prefix(sym).and_then(prefixable_symbol) {
            return Some((exp, dim));
        }
        if let Some(dim) = name.strip_prefix(long).and_then(prefixable_long) {
            return Some((exp, dim));
        }
    }
    None
}

/// Dimension of a unit name with optional `SI::` and SI prefix.
fn unit_lookup(name: &str) -> Option<Dim> {
    split_si_prefix(name).map(|(_, dim)| dim)
}

/// Dimension of a `typedBy:` quantity-type reference (`ISQ::MassValue`, or a bare name).
pub fn quantity_dimension(s: &str) -> Option<Dim> {
    quantity_table(s.strip_prefix("ISQ::").unwrap_or(s))
}

/// Dimension of a `unit:` reference (`SI::kilogram`, `SI::kg`, or a bare symbol `kg`), or of a
/// compound unit expression built from table units with `*`, `/` and `^` (`m/s`, `N*m`, `m^2`,
/// `kg*m/s^2`), evaluated left to right (REQ-TRS-SYSMLV2-064).
pub fn unit_dimension(s: &str) -> Option<Dim> {
    let s = s.trim();
    if let Some(d) = unit_lookup(s.strip_prefix("SI::").unwrap_or(s)) {
        return Some(d);
    }
    if !s.contains(['*', '/', '^']) {
        return None;
    }
    let mut acc = Dim([0; 7]);
    let mut op = '*';
    let mut term = String::new();
    let flush = |acc: &mut Dim, op: char, term: &str| -> Option<()> {
        let (base, exp) = match term.split_once('^') {
            Some((b, e)) => (b.trim(), e.trim().parse::<i8>().ok()?),
            None => (term.trim(), 1),
        };
        let td = if base == "1" { Dim([0; 7]) } else { unit_lookup(base.strip_prefix("SI::").unwrap_or(base))? };
        let sign = if op == '*' { 1i8 } else { -1i8 };
        for (a, t) in acc.0.iter_mut().zip(td.0) {
            *a = a.checked_add(sign.checked_mul(t.checked_mul(exp)?)?)?;
        }
        Some(())
    };
    for c in s.chars() {
        if c == '*' || c == '/' {
            flush(&mut acc, op, &term)?;
            op = c;
            term.clear();
        } else {
            term.push(c);
        }
    }
    flush(&mut acc, op, &term)?;
    Some(acc)
}

/// Whether `s` is a recognised built-in type reference, for `W404` suppression
/// (REQ-TRS-LIB-002): a closed-package member (`ScalarValues`/`Base`), an `ISQ`
/// quantity-value type, or an `SI` unit.
pub fn is_recognised_type_ref(s: &str) -> bool {
    crate::resolver::is_builtin_type(s)
        || quantity_dimension(s).is_some()
        || unit_dimension(s).is_some()
}

/// ISQ quantity names with no dimension recorded here (so no `W044` check) but
/// that exist in the SysML v2 quantities library; used only to keep `W057` from
/// flagging real names.
const ISQ_EXTRA: &[&str] = &[
    "ActivityValue", "AbsorbedDoseValue", "AngularMomentumValue", "AngularFrequencyValue",
    "AngularMeasureValue", "AreaDensityValue", "BreadthValue", "CartesianPosition3dVector",
    "CartesianSpatial3dCoordinateSystem", "ConductanceValue", "CurrentDensityValue",
    "DiameterValue", "DynamicViscosityValue", "ElectricChargeDensityValue",
    "ElectricConductanceValue", "ElectricFieldStrengthValue", "ElectricPowerValue",
    "EnergyDensityValue", "EntropyValue", "HeatCapacityValue", "HeightValue",
    "IlluminanceValue", "InductanceValue", "KinematicViscosityValue", "LengthPerTimeValue",
    "LuminanceValue", "LuminousFluxValue", "MagneticFieldStrengthValue",
    "MagneticFluxDensityValue", "MagneticFluxValue", "MassPerLengthValue",
    "MolarMassValue", "MomentOfInertiaValue", "PeriodValue", "PermeabilityValue",
    "PermittivityValue", "Position3dVector", "PositionVector", "PowerDensityValue",
    "RadiusValue", "RotationalFrequencyValue", "SpecificEnergyValue",
    "SpecificHeatCapacityValue", "SpecificVolumeValue", "SurfaceTensionValue",
    "TensorMeasurementReference", "ThermalConductivityValue", "ThicknessValue",
    "VelocityVector", "VolumetricFlowRateValue", "WavelengthValue", "WidthValue",
    "WorkValue", "PathLengthValue", "DistanceValue", "TemperatureDifferenceValue",
    "CelsiusTemperatureValue", "DisplacementVector", "AccelerationVector", "ForceVector",
    "ElectricCurrentDensityValue", "MassFractionValue", "RelativeValue",
];

/// SI unit names/symbols that exist in the SI library but carry no dimension here.
const SI_EXTRA: &[&str] = &[
    "becquerel", "Bq", "gray", "Gy", "sievert", "Sv", "lumen", "lm", "lux", "lx", "katal",
    "kat", "microampere", "uA", "nanosecond", "ns", "picosecond", "ps", "micrometre", "um",
    "nanometre", "nm", "gigahertz", "GHz", "kilopascal", "megapascal", "hectopascal", "hPa",
    "milliwatt", "mW", "gigawatt", "GW", "kilovoltAmpere", "VA", "millijoule", "mJ",
    "megajoule", "MJ", "milliohm", "kiloohm", "kOhm", "megaohm", "MOhm", "microfarad", "uF",
    "nanofarad", "nF", "picofarad", "pF", "millihenry", "mH", "microhenry", "uH", "day",
    "d", "week", "year", "arcminute", "arcsecond", "hectare", "ha", "degreeFahrenheit",
    "Mbps", "kbps", "Gbps", "bps", "byte", "B", "bit", "kilobyte", "kB", "megabyte", "MB",
    "gigabyte", "GB", "knot", "kn", "metrePerSecondSquared", "g_n", "Ohm", "ohmMetre",
    "siemensPerMetre", "S", "T", "mT", "uT", "Ah", "mAh", "ampereHour", "milliampereHour",
    "kilometrePerHour", "m2", "m3", "cm2", "cm3", "mm2", "mm3", "lbf", "ft", "in",
];

use crate::stdlib_names::{ISQ_NAMES, SI_NAMES, SI_PREFIX_NAMES, US_CUSTOMARY_NAMES};

/// Whether `m` is a package-level member of the ISQ quantities library: the complete
/// inventory extracted from the standard library plus this build's curated tables
/// (a superset, so a name the tool already recognised never starts being rejected).
fn isq_member(m: &str) -> bool {
    quantity_table(m).is_some() || ISQ_EXTRA.contains(&m) || ISQ_NAMES.contains(&m)
}

/// Whether `m` is a member of the `SI` package: its own units, plus the `ISQ::*` and
/// `SIPrefixes::*` it publicly re-exports.
fn si_member(m: &str) -> bool {
    unit_table(m).is_some()
        || SI_EXTRA.contains(&m)
        || SI_NAMES.contains(&m)
        || SI_PREFIX_NAMES.contains(&m)
        || isq_member(m)
}

/// Whether `member` is declared by the library package `pkg` (GH #210). `Some(bool)` for
/// a package whose full membership is known here (`ISQ` and its `ISQ*` sub-packages,
/// `SI`, `SIPrefixes`, `USCustomaryUnits`, and the closed `ScalarValues`/`Base`);
/// `None` for any other package, whose membership is not enumerated.
pub fn library_member_known(pkg: &str, member: &str) -> Option<bool> {
    use crate::resolver::BUILTIN_TYPE_PACKAGES;
    if let Some((_, members)) = BUILTIN_TYPE_PACKAGES.iter().find(|(p, _)| *p == pkg) {
        return Some(members.contains(&member));
    }
    Some(match pkg {
        "ISQ" | "ISQBase" | "ISQSpaceTime" | "ISQMechanics" | "ISQThermodynamics"
        | "ISQElectromagnetism" | "ISQLight" | "ISQAcoustics" | "ISQChemistryMolecular"
        | "ISQAtomicNuclear" | "ISQCondensedMatter" | "ISQCharacteristicNumbers"
        | "ISQInformation" => isq_member(member),
        "SI" => si_member(member),
        "SIPrefixes" => SI_PREFIX_NAMES.contains(&member),
        "USCustomaryUnits" => US_CUSTOMARY_NAMES.contains(&member) || si_member(member),
        _ => return None,
    })
}

/// `Some(true)` when `ISQ::<member>` is a name this build knows, `Some(false)`
/// when it is not; `None` when `s` is not an `ISQ::` reference.
pub fn isq_name_known(s: &str) -> Option<bool> {
    let m = s.strip_prefix("ISQ::")?;
    if m.contains("::") {
        return None;
    }
    Some(isq_member(m))
}

/// As [`isq_name_known`], for `SI::<unit>` (a bare unit symbol is not judged).
pub fn si_unit_known(s: &str) -> Option<bool> {
    let m = s.trim().strip_prefix("SI::")?;
    if m.contains(['*', '/', '^', ' ']) || m.contains("::") {
        return None;
    }
    Some(si_member(m) || unit_lookup(m).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dim(s: &str) -> Option<Dim> {
        unit_dimension(s)
    }

    #[test]
    fn prefixed_symbols_resolve_to_the_base_dimension() {
        let force = dim("N").unwrap();
        for u in ["kN", "MN", "mN", "µN", "uN", "nN", "GN", "SI::kN"] {
            assert_eq!(dim(u), Some(force), "{u}");
        }
        assert_eq!(dim("mm"), dim("m"));
        assert_eq!(dim("MHz"), dim("Hz"));
        assert_eq!(dim("GHz"), dim("Hz"));
        assert_eq!(dim("nF"), dim("F"));
        assert_eq!(dim("kOhm"), dim("ohm"));
        assert_eq!(dim("kΩ"), dim("ohm"));
        assert_eq!(dim("hPa"), dim("Pa"));
        assert_eq!(dim("daN"), dim("N"));
        assert_eq!(dim("mol"), Some(Dim([0, 0, 0, 0, 0, 1, 0])));
        assert_eq!(dim("mmol"), dim("mol"));
        assert_eq!(dim("kcd"), dim("cd"));
    }

    #[test]
    fn prefixed_long_names_resolve() {
        assert_eq!(dim("SI::kilonewton"), dim("N"));
        assert_eq!(dim("megahertz"), dim("Hz"));
        assert_eq!(dim("micrometre"), dim("m"));
        assert_eq!(dim("nanosecond"), dim("s"));
        assert_eq!(dim("gigapascal"), dim("Pa"));
        // a symbol prefix on a long base (or vice versa) is not a unit
        assert_eq!(dim("kNewton"), None);
        assert_eq!(dim("kilon"), None);
    }

    #[test]
    fn ambiguous_strings_keep_their_table_meaning() {
        // `min` is minutes (time), not milli-inch; `m` is the metre, not a bare prefix.
        assert_eq!(dim("min"), dim("s"));
        assert_eq!(dim("m"), Some(Dim([1, 0, 0, 0, 0, 0, 0])));
        assert_eq!(dim("Pa"), Some(Dim([-1, 1, -2, 0, 0, 0, 0])));
        assert_eq!(dim("cd"), Some(Dim([0, 0, 0, 0, 0, 0, 1])));
        assert_eq!(dim("t"), dim("kg"));
    }

    #[test]
    fn non_prefixable_and_bogus_units_are_rejected() {
        for u in ["kdegC", "krpm", "mmin", "zorp", "k", "kk", "SI::kzorp"] {
            assert_eq!(dim(u), None, "{u}");
        }
    }

    #[test]
    fn prefix_exponents() {
        assert_eq!(split_si_prefix("kN").map(|x| x.0), Some(3));
        assert_eq!(split_si_prefix("µs").map(|x| x.0), Some(-6));
        assert_eq!(split_si_prefix("GHz").map(|x| x.0), Some(9));
        assert_eq!(split_si_prefix("picofarad").map(|x| x.0), Some(-12));
        assert_eq!(split_si_prefix("N").map(|x| x.0), Some(0));
    }

    #[test]
    fn compound_expressions_accept_prefixed_terms() {
        assert_eq!(dim("kN*m"), dim("Nm"));
        assert_eq!(dim("mm/ms"), dim("m/s"));
        assert_eq!(dim("MHz*s"), Some(Dim([0; 7])));
        assert_eq!(dim("km/h"), dim("m/s"));
        assert_eq!(dim("mm^2"), Some(Dim([2, 0, 0, 0, 0, 0, 0])));
    }

    #[test]
    fn si_unit_known_accepts_prefixed_but_not_typos() {
        assert_eq!(si_unit_known("SI::kN"), Some(true));
        assert_eq!(si_unit_known("SI::terahertz"), Some(true));
        assert_eq!(si_unit_known("SI::kNN"), Some(false));
        assert_eq!(si_unit_known("SI::zorp"), Some(false));
        assert_eq!(si_unit_known("kN"), None);
    }
}
