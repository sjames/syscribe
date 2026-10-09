//! Heat-table output shared by `fmea report --format`, `cyber-risk --format` and
//! `hara matrix` (GH #223). The grids themselves live in `syscribe_model::heat`.

use syscribe_model::element::RawElement;
use syscribe_model::heat::{self, HeatGrid};
use syscribe_model::risk::CyberConfig;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HeatFormat {
    Md,
    Html,
    Json,
}

/// `Some(format)` when `--format md|html|json` is present (exits on a bad value),
/// `None` when absent.
pub fn parse_format(cmd: &str, args: &[String]) -> Option<HeatFormat> {
    if !args.iter().any(|a| a == "--format") {
        return None;
    }
    let v = crate::cliargs::or_exit(crate::cliargs::enum_value(cmd, args, "--format", &["md", "html", "json"], "md"));
    Some(match v {
        "html" => HeatFormat::Html,
        "json" => HeatFormat::Json,
        _ => HeatFormat::Md,
    })
}

pub fn render(grid: &HeatGrid, fmt: HeatFormat) -> String {
    match fmt {
        HeatFormat::Md => grid.to_markdown(),
        HeatFormat::Html => heat::html_page(&grid.title, std::slice::from_ref(grid)),
        HeatFormat::Json => format!("{}\n", serde_json::to_string_pretty(&grid.to_json()).unwrap_or_default()),
    }
}

pub fn emit(grid: &HeatGrid, fmt: HeatFormat) {
    print!("{}", render(grid, fmt));
}

/// `hara matrix [--format md|html|json]`.
pub fn cmd_hara(elements: &[RawElement], sub: &str, rest: &[String]) -> i32 {
    match sub {
        "matrix" => {
            let fmt = parse_format("hara", rest).unwrap_or(HeatFormat::Md);
            emit(&heat::hara_grid(elements), fmt);
            0
        }
        _ => {
            eprintln!("Usage: syscribe -m <model> hara matrix [--format md|html|json]");
            1
        }
    }
}

/// `cyber-risk --format ...` heat view.
pub fn cyber_heat(elements: &[RawElement], cfg: &CyberConfig, fmt: HeatFormat) {
    emit(&heat::tara_grid(elements, cfg), fmt);
}
