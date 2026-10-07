/// Colour theme for one element type, as CSS hex strings.
#[derive(Debug, Clone)]
pub struct ElementTheme {
    pub header_bg: &'static str,
    pub header_fg: &'static str,
    pub body_bg: &'static str,
    pub body_fg: &'static str,
    pub border: &'static str,
}
