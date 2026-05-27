use ab_glyph::{Font, FontArc, PxScale, ScaleFont};

pub trait TextMetrics: Send + Sync {
    fn advance_width(&self, text: &str, font_size: f64, bold: bool) -> f64;
    fn line_height(&self, font_size: f64) -> f64 {
        font_size * 1.35
    }
    fn cap_height(&self, font_size: f64) -> f64 {
        font_size * 0.72
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
        text.chars()
            .map(|c| scaled.h_advance(scaled.glyph_id(c)) as f64)
            .sum()
    }
}

impl TextMetrics for FontMetrics {
    fn advance_width(&self, text: &str, font_size: f64, bold: bool) -> f64 {
        self.measure(text, font_size, bold)
    }
}

/// Fallback: 0.58× font_size per character (reasonable for Roboto/Inter).
pub struct ApproxMetrics;

impl TextMetrics for ApproxMetrics {
    fn advance_width(&self, text: &str, font_size: f64, _bold: bool) -> f64 {
        text.chars().count() as f64 * font_size * 0.58
    }
}

/// Try to load Roboto/NotoSans/DejaVu from the system, fall back to ApproxMetrics.
pub fn load_metrics() -> Box<dyn TextMetrics> {
    if let Some(m) = try_load_font_metrics() {
        Box::new(m)
    } else {
        Box::new(ApproxMetrics)
    }
}

fn try_load_font_metrics() -> Option<FontMetrics> {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let regular_bytes = find_font_bytes(&db, fontdb::Weight::NORMAL)?;
    let bold_bytes = find_font_bytes(&db, fontdb::Weight::BOLD)
        .unwrap_or_else(|| regular_bytes.clone());

    let regular = FontArc::try_from_vec(regular_bytes).ok()?;
    let bold = FontArc::try_from_vec(bold_bytes).ok()?;

    Some(FontMetrics { regular, bold })
}

fn find_font_bytes(db: &fontdb::Database, weight: fontdb::Weight) -> Option<Vec<u8>> {
    let id = db.query(&fontdb::Query {
        families: &[
            fontdb::Family::Name("Roboto"),
            fontdb::Family::Name("Inter"),
            fontdb::Family::Name("Noto Sans"),
            fontdb::Family::Name("DejaVu Sans"),
            fontdb::Family::SansSerif,
        ],
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
