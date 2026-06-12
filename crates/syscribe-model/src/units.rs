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
        "radian" | "rad" | "steradian" | "sr" => d(0, 0, 0, 0, 0, 0, 0),
        "litre" | "liter" | "L" | "millilitre" | "mL" => d(3, 0, 0, 0, 0, 0, 0),
        "metrePerSecond" | "mps" | "kilometrePerHour" | "kph" | "kmh" => d(1, 0, -1, 0, 0, 0, 0),
        _ => return None,
    })
}

/// Dimension of a `typedBy:` quantity-type reference (`ISQ::MassValue`, or a bare name).
pub fn quantity_dimension(s: &str) -> Option<Dim> {
    quantity_table(s.strip_prefix("ISQ::").unwrap_or(s))
}

/// Dimension of a `unit:` reference (`SI::kilogram`, `SI::kg`, or a bare symbol `kg`).
pub fn unit_dimension(s: &str) -> Option<Dim> {
    unit_table(s.strip_prefix("SI::").unwrap_or(s))
}

/// Whether `s` is a recognised built-in type reference, for `W404` suppression
/// (REQ-TRS-LIB-002): a closed-package member (`ScalarValues`/`Base`), an `ISQ`
/// quantity-value type, or an `SI` unit.
pub fn is_recognised_type_ref(s: &str) -> bool {
    crate::resolver::is_builtin_type(s)
        || quantity_dimension(s).is_some()
        || unit_dimension(s).is_some()
}
