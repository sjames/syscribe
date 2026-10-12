//! Text metrics for sizing diagram content (`REQ-TRS-VIS-017`), promoted
//! from the CLI's `svgkit` where the MagicGrid report (`REQ-TRS-MG-016`)
//! first used them. Two implementations of one trait:
//!
//! - [`FontMetrics`]: real glyph advances from a system font found through
//!   `fontdb` and measured with `ab_glyph`;
//! - [`ApproxMetrics`]: a per-character estimate used when no suitable
//!   system font is installed (containers, CI images without fonts);
//! - [`BundledMetrics`]: the advance widths of Liberation Sans compiled into
//!   the binary (the generated `metrics_table` module), the same on every machine.
//!
//! [`load_metrics`] keeps the family stack the MagicGrid report has always
//! used (Roboto, Inter, Noto Sans, DejaVu Sans, any sans), so its SVG output
//! is unchanged by the move. [`diagram_metrics`] is what the diagram sizer
//! ([`super::size`]) uses: [`BundledMetrics`], never the installed fonts, so
//! the same model gives byte-identical diagrams on every machine (GH #339).
//! Liberation Sans is metric-compatible with Arial and Helvetica, the head of
//! the `font-family` stack the static SVG writer names and the browser's CSS
//! must match (`ADR-SYS-VIS-001`, addendum); the sizer adds a safety margin on
//! top so a slightly wider browser font never clips a label.

use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

pub trait TextMetrics: Send + Sync {
    /// The advance width of `text` set at `font_size` pixels.
    fn advance_width(&self, text: &str, font_size: f64, bold: bool) -> f64;
    /// A line box for `font_size` pixels.
    fn line_height(&self, font_size: f64) -> f64 {
        font_size * 1.35
    }
}

/// ab_glyph-backed metrics using system fonts discovered via fontdb.
pub struct FontMetrics {
    regular: FontArc,
    bold: FontArc,
}

impl FontMetrics {
    fn measure(&self, text: &str, font_size: f64, bold: bool) -> f64 {
        let font = if bold { &self.bold } else { &self.regular };
        let scale = PxScale::from(font_size as f32);
        let scaled = font.as_scaled(scale);
        text.chars().map(|c| scaled.h_advance(scaled.glyph_id(c)) as f64).sum()
    }
}

impl TextMetrics for FontMetrics {
    fn advance_width(&self, text: &str, font_size: f64, bold: bool) -> f64 {
        self.measure(text, font_size, bold)
    }
}

/// Fallback: 0.58× font_size per character (reasonable for Roboto/Inter and
/// a touch generous for Helvetica/Arial, which is the safe direction).
pub struct ApproxMetrics;

impl TextMetrics for ApproxMetrics {
    fn advance_width(&self, text: &str, font_size: f64, _bold: bool) -> f64 {
        text.chars().count() as f64 * font_size * 0.58
    }
}

/// The advance widths of Liberation Sans, compiled in (GH #339): no font file is read, so the result does not
/// depend on what the machine has installed. A character outside the table is measured as [`ApproxMetrics`] does.
pub struct BundledMetrics;

impl BundledMetrics {
    fn advance(c: char, bold: bool) -> Option<f64> {
        let table = super::metrics_table::ADVANCES;
        let i = table.binary_search_by(|(k, _, _)| k.cmp(&c)).ok()?;
        let (_, r, b) = table[i];
        Some(f64::from(if bold { b } else { r }) / super::metrics_table::UNITS_PER_EM)
    }
}

impl TextMetrics for BundledMetrics {
    fn advance_width(&self, text: &str, font_size: f64, bold: bool) -> f64 {
        text.chars().map(|c| Self::advance(c, bold).unwrap_or(0.58) * font_size).sum()
    }
}

/// The MagicGrid report's family stack (unchanged since `svgkit`).
pub const REPORT_FAMILIES: &[&str] = &["Roboto", "Inter", "Noto Sans", "DejaVu Sans"];

/// The diagram family stack: the `font-family` the static SVG writer emits
/// and the browser client's CSS must use.
pub const DIAGRAM_FAMILIES: &[&str] = &["Helvetica", "Arial", "Liberation Sans", "DejaVu Sans"];

/// Try to load Roboto/Inter/Noto Sans/DejaVu Sans from the system, fall back
/// to [`ApproxMetrics`] — the MagicGrid report's metrics.
pub fn load_metrics() -> Box<dyn TextMetrics> {
    load_metrics_for(REPORT_FAMILIES)
}

/// Load the first installed family of `families` (then any sans-serif face),
/// falling back to [`ApproxMetrics`] when fontdb finds none or the face
/// cannot be parsed.
pub fn load_metrics_for(families: &[&str]) -> Box<dyn TextMetrics> {
    match try_load_font_metrics(families) {
        Some(m) => Box::new(m),
        None => Box::new(ApproxMetrics),
    }
}

/// The diagram metrics: [`BundledMetrics`], whatever fonts are installed (GH #339), so a diagram exported here
/// and one exported on a machine with other fonts, or none, are the same file.
pub fn diagram_metrics() -> &'static dyn TextMetrics {
    static METRICS: BundledMetrics = BundledMetrics;
    &METRICS
}

fn try_load_font_metrics(families: &[&str]) -> Option<FontMetrics> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let regular_bytes = find_font_bytes(&db, families, fontdb::Weight::NORMAL)?;
    let bold_bytes = find_font_bytes(&db, families, fontdb::Weight::BOLD).unwrap_or_else(|| regular_bytes.clone());

    let regular = FontArc::try_from_vec(regular_bytes).ok()?;
    let bold = FontArc::try_from_vec(bold_bytes).ok()?;

    Some(FontMetrics { regular, bold })
}

fn find_font_bytes(db: &fontdb::Database, families: &[&str], weight: fontdb::Weight) -> Option<Vec<u8>> {
    let mut query_families: Vec<fontdb::Family> = families.iter().map(|f| fontdb::Family::Name(f)).collect();
    query_families.push(fontdb::Family::SansSerif);
    let id = db.query(&fontdb::Query {
        families: &query_families,
        weight,
        style: fontdb::Style::Normal,
        stretch: fontdb::Stretch::Normal,
    })?;

    let face = db.face(id)?;
    match &face.source {
        fontdb::Source::File(path) => std::fs::read(path).ok(),
        fontdb::Source::Binary(data) => Some(data.as_ref().as_ref().to_vec()),
        fontdb::Source::SharedFile(path, _) => std::fs::read(path).ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approx_metrics_scale_with_length_and_size() {
        let m = ApproxMetrics;
        assert_eq!(m.advance_width("abcd", 10.0, false), 4.0 * 5.8);
        assert!(m.advance_width("abcd", 12.0, true) > m.advance_width("abcd", 10.0, true));
        assert_eq!(m.advance_width("", 12.0, false), 0.0);
        assert!((m.line_height(10.0) - 13.5).abs() < 1e-9);
    }

    #[test]
    fn bundled_metrics_are_liberation_sans_and_fall_back_per_character() {
        let m = BundledMetrics;
        // 'M' is 1706/2048 em regular, 1706 bold; 'i' 455 / 569
        assert!((m.advance_width("M", 2048.0, false) - 1706.0).abs() < 1e-9);
        assert!((m.advance_width("i", 2048.0, true) - 569.0).abs() < 1e-9);
        assert!(m.advance_width("Wide", 12.0, true) > m.advance_width("Wide", 12.0, false));
        // a character outside the table is measured as ApproxMetrics does
        assert!((m.advance_width("日", 10.0, false) - 5.8).abs() < 1e-9);
        // the table is sorted, so every entry is found
        let t = super::super::metrics_table::ADVANCES;
        assert!(t.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn the_diagram_sizer_measures_with_the_bundled_table_not_the_installed_fonts() {
        // whatever this machine has installed, the sizer's widths are the table's
        for (text, bold) in [("Grid-Tie Inverter", true), ("«part def»", false), ("pvIn: DcPowerPort", false)] {
            assert_eq!(diagram_metrics().advance_width(text, 12.0, bold), BundledMetrics.advance_width(text, 12.0, bold));
        }
    }

    #[test]
    fn loaded_metrics_measure_monotonically_whatever_font_is_installed() {
        let m = diagram_metrics();
        let short = m.advance_width("Engine", 12.0, true);
        let long = m.advance_width("Engine : PowerSystem", 12.0, true);
        assert!(short > 0.0 && long > short);
        assert!(m.advance_width("Engine", 12.0, true) >= m.advance_width("Engine", 9.0, true));
        // The same instance every time.
        assert!(std::ptr::eq(diagram_metrics(), m));
    }
}
