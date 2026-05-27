/// Route an edge between two absolute port anchor points.
/// Returns an SVG `<path>` element string.
/// For MVP: 3-segment orthogonal routing (horizontal → vertical → horizontal).
pub fn route_edge(
    x1: f64, y1: f64,   // source anchor (absolute)
    x2: f64, y2: f64,   // target anchor (absolute)
    kind: &str,
) -> String {
    let (stroke, dash, marker_id) = edge_style(kind);
    let dash_attr = if dash.is_empty() {
        String::new()
    } else {
        format!(" stroke-dasharray=\"{}\"", dash)
    };

    let mid_x = (x1 + x2) / 2.0;

    // 3-segment: exit horizontally from source, join midpoint, enter target
    let d = if (y1 - y2).abs() < 2.0 {
        // Straight horizontal line
        format!("M {x1:.1} {y1:.1} L {x2:.1} {y2:.1}")
    } else {
        format!(
            "M {x1:.1} {y1:.1} L {mx:.1} {y1:.1} L {mx:.1} {y2:.1} L {x2:.1} {y2:.1}",
            x1 = x1, y1 = y1, mx = mid_x, y2 = y2, x2 = x2
        )
    };

    let label = edge_label(kind);
    let label_x = mid_x;
    let label_y = (y1 + y2) / 2.0 - 4.0;
    let label_svg = if label.is_empty() {
        String::new()
    } else {
        format!(
            "<text x=\"{lx:.1}\" y=\"{ly:.1}\" font-size=\"9\" fill=\"{stroke}\" \
             text-anchor=\"middle\" font-style=\"italic\">«{label}»</text>",
            lx = label_x, ly = label_y, stroke = stroke, label = label
        )
    };

    format!(
        "<path d=\"{d}\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"1.4\"\
         {dash} marker-end=\"url(#{marker})\"/>{label}",
        d = d, stroke = stroke, dash = dash_attr, marker = marker_id, label = label_svg
    )
}

fn edge_style(kind: &str) -> (&'static str, &'static str, &'static str) {
    match kind {
        "flow" => ("#3a6ea5", "", "arr-flow"),
        "derive" | "derivedFrom" => ("#555", "5,3", "arr-open"),
        "verify" | "verifies" => ("#3a6ea5", "5,3", "arr-verify"),
        "allocate" | "allocatedTo" => ("#7a3ea5", "3,3", "arr-alloc"),
        "satisfy" | "satisfies" => ("#1a6b30", "4,2", "arr-satisfy"),
        "generalize" | "supertype" => ("#3a3a5c", "", "arr-generalize"),
        "use" => ("#888", "6,3", "arr-open"),
        _ => ("#888", "", "arr-open"),
    }
}

fn edge_label(kind: &str) -> &'static str {
    match kind {
        "derive" | "derivedFrom" => "derive",
        "verify" | "verifies" => "verify",
        "allocate" | "allocatedTo" => "allocate",
        "satisfy" | "satisfies" => "satisfy",
        "use" => "use",
        _ => "",
    }
}

/// SVG `<defs>` block containing all arrowhead markers.
pub fn arrowhead_defs() -> String {
    let entries: &[(&str, &str, bool)] = &[
        ("arr-open", "#888", false),
        ("arr-flow", "#3a6ea5", false),
        ("arr-verify", "#3a6ea5", false),
        ("arr-alloc", "#7a3ea5", false),
        ("arr-satisfy", "#1a6b30", false),
        ("arr-generalize", "#3a3a5c", true),  // hollow triangle for generalization
    ];

    let mut out = String::from("<defs>");
    for (id, color, hollow) in entries {
        if *hollow {
            out.push_str(&format!(
                "<marker id=\"{id}\" markerWidth=\"12\" markerHeight=\"10\" \
                 refX=\"11\" refY=\"5\" orient=\"auto\">\
                 <polygon points=\"0,0 12,5 0,10\" fill=\"white\" \
                 stroke=\"{c}\" stroke-width=\"1.2\"/></marker>",
                id = id, c = color
            ));
        } else {
            out.push_str(&format!(
                "<marker id=\"{id}\" markerWidth=\"10\" markerHeight=\"7\" \
                 refX=\"9\" refY=\"3.5\" orient=\"auto\">\
                 <polyline points=\"0,0 9,3.5 0,7\" fill=\"none\" \
                 stroke=\"{c}\" stroke-width=\"1.2\"/></marker>",
                id = id, c = color
            ));
        }
    }
    out.push_str("</defs>");
    out
}
