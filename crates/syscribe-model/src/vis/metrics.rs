//! Text metrics for sizing diagram content (`REQ-TRS-VIS-017`), promoted
//! from the CLI's `svgkit` where the MagicGrid report (`REQ-TRS-MG-016`)
//! first used them. Two implementations of one trait:
//!
//! - [`FontMetrics`]: real glyph advances from a system font found through
//!   `fontdb` and measured with `ab_glyph`;
//! - [`ApproxMetrics`]: a per-character estimate used when no suitable
//!   system font is installed (containers, CI images without fonts).
//!
//! [`load_metrics`] keeps the family stack the MagicGrid report has always
//! used (Roboto, Inter, Noto Sans, DejaVu Sans, any sans), so its SVG output
//! is unchanged by the move. [`diagram_metrics`] is the process-wide instance
//! the diagram sizer ([`super::size`]) uses: the Helvetica → Arial →
//! Liberation Sans → DejaVu Sans → any-sans stack that the static SVG writer
//! names in its `font-family` and that the browser's CSS stack must match
//! (`ADR-SYS-VIS-001`, addendum): the sizer adds a safety margin on top so a
//! slightly wider browser font never clips a label.

use std::sync::OnceLock;

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

/// The process-wide diagram metrics ([`DIAGRAM_FAMILIES`]), loaded on first
/// use. System-font discovery costs tens of milliseconds, so every sizer
/// call shares one instance.
pub fn diagram_metrics() -> &'static dyn TextMetrics {
    static METRICS: OnceLock<Box<dyn TextMetrics>> = OnceLock::new();
    METRICS.get_or_init(|| load_metrics_for(DIAGRAM_FAMILIES)).as_ref()
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
