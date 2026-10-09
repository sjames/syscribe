//! ISO 26262 ASIL arithmetic (GH #214, #215): level ranking, decomposition
//! pair legality (ISO 26262-9 §5), the `X(Y)` decomposition notation, and the
//! HARA S/E/C -> ASIL determination table (ISO 26262-3 Table 4).

/// Valid `asilLevel:` values.
pub const ASIL_VALUES: &[&str] = &["QM", "A", "B", "C", "D"];

/// Rank of an ASIL: QM = 0, A = 1 ... D = 4.
pub fn rank(level: &str) -> Option<u8> {
    match level.trim().to_ascii_uppercase().as_str() {
        "QM" => Some(0),
        "A" => Some(1),
        "B" => Some(2),
        "C" => Some(3),
        "D" => Some(4),
        _ => None,
    }
}

/// Name for a rank (inverse of [`rank`]).
pub fn name(rank: u8) -> &'static str {
    match rank {
        0 => "QM",
        1 => "A",
        2 => "B",
        3 => "C",
        _ => "D",
    }
}

/// Split the ISO 26262-9 decomposition notation `B(D)` into the effective level
/// (`B`) and the original level (`D`). Any other value is returned unchanged
/// (so a malformed value still reaches `E010`).
pub fn split_notation(v: Option<String>) -> (Option<String>, Option<String>) {
    let Some(s) = v else { return (None, None) };
    let t = s.trim();
    if let Some(open) = t.find('(') {
        if t.ends_with(')') {
            let base = t[..open].trim();
            let orig = t[open + 1..t.len() - 1].trim();
            if rank(base).is_some() && matches!(rank(orig), Some(1..=4)) {
                return (
                    Some(base.to_ascii_uppercase()),
                    Some(orig.to_ascii_uppercase()),
                );
            }
        }
    }
    (Some(s), None)
}

/// Whether two decomposed channels `a`, `b` (ranks, QM = 0) are an allowed
/// decomposition of `parent` (ISO 26262-9 Table 1): D = C+A | B+B | D+QM,
/// C = B+A | C+QM, B = A+A | B+QM, A = A+QM. Stronger pairs are accepted;
/// each channel is capped at the parent level.
pub fn pair_legal(parent: u8, a: u8, b: u8) -> bool {
    parent == 0 || a.min(parent) + b.min(parent) >= parent
}

/// ASIL determination for a hazardous event (ISO 26262-3 Table 4) from
/// severity `S0..S3`, exposure `E0..E4`, controllability `C0..C3`.
/// Returns `None` if any value is unparseable.
pub fn derive(severity: &str, exposure: &str, controllability: &str) -> Option<&'static str> {
    let num = |s: &str, p: char, max: u8| -> Option<u8> {
        let rest = s.trim().strip_prefix(p)?;
        let n: u8 = rest.parse().ok()?;
        (n <= max).then_some(n)
    };
    let s = num(severity, 'S', 3)?;
    let e = num(exposure, 'E', 4)?;
    let c = num(controllability, 'C', 3)?;
    if s == 0 || e == 0 || c == 0 {
        return Some("QM");
    }
    Some(match s + e + c {
        10 => "D",
        9 => "C",
        8 => "B",
        7 => "A",
        _ => "QM",
    })
}

/// Parse a fault-tolerant-time-interval string (`50ms`, `1.5 s`, `2min`, `1h`,
/// `200us`) into milliseconds. `None` when it is not `<number><unit>`.
pub fn ftti_millis(s: &str) -> Option<f64> {
    let t = s.trim();
    let split = t.find(|c: char| !(c.is_ascii_digit() || c == '.'))?;
    let (num, unit) = t.split_at(split);
    let n: f64 = num.parse().ok()?;
    // Trailing commentary after the value is tolerated (`200 ms — bounded by …`):
    // the unit is the leading alphabetic run and must not run into more letters.
    let unit = unit.trim_start();
    let end = unit.find(|c: char| !(c.is_alphabetic() || c == '\u{b5}')).unwrap_or(unit.len());
    let (unit, rest) = unit.split_at(end);
    if rest.starts_with(|c: char| c.is_alphanumeric()) {
        return None;
    }
    let factor = match unit {
        "ns" => 1e-6,
        "us" | "\u{b5}s" => 1e-3,
        "ms" => 1.0,
        "s" => 1e3,
        "min" => 6e4,
        "h" => 3.6e6,
        _ => return None,
    };
    Some(n * factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_matches_iso() {
        assert_eq!(derive("S3", "E4", "C3"), Some("D"));
        assert_eq!(derive("S3", "E4", "C2"), Some("C"));
        assert_eq!(derive("S1", "E4", "C3"), Some("B"));
        assert_eq!(derive("S1", "E3", "C3"), Some("A"));
        assert_eq!(derive("S2", "E3", "C2"), Some("A"));
        assert_eq!(derive("S0", "E4", "C3"), Some("QM"));
        assert_eq!(derive("S3", "E1", "C1"), Some("QM"));
        assert_eq!(derive("S3", "E9", "C1"), None);
    }

    #[test]
    fn pairs() {
        assert!(pair_legal(4, 3, 1) && pair_legal(4, 2, 2) && pair_legal(4, 4, 0));
        assert!(!pair_legal(4, 1, 1) && !pair_legal(4, 2, 1));
        assert!(pair_legal(3, 2, 1) && pair_legal(3, 3, 0) && !pair_legal(3, 1, 1));
        assert!(pair_legal(2, 1, 1) && pair_legal(2, 2, 0) && !pair_legal(2, 1, 0));
        assert!(pair_legal(1, 1, 0) && !pair_legal(1, 0, 0));
    }

    #[test]
    fn notation() {
        assert_eq!(
            split_notation(Some("D(D)".into())),
            (Some("D".into()), Some("D".into()))
        );
        assert_eq!(
            split_notation(Some("QM(D)".into())),
            (Some("QM".into()), Some("D".into()))
        );
        assert_eq!(split_notation(Some("X(D)".into())), (Some("X(D)".into()), None));
        assert_eq!(split_notation(Some("B".into())), (Some("B".into()), None));
    }

    #[test]
    fn ftti() {
        assert_eq!(ftti_millis("50ms"), Some(50.0));
        assert_eq!(ftti_millis("1 s"), Some(1000.0));
        assert!(ftti_millis("banana").is_none());
        assert!(ftti_millis("50").is_none());
        assert!(ftti_millis("5 parsecs").is_none());
        assert_eq!(ftti_millis("200 ms — bounded by the ESD circuit"), Some(200.0));
        assert_eq!(ftti_millis("10 ms, at 100 Hz tick"), Some(10.0));
    }
}
