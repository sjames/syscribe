/// Public edge style descriptor.
pub struct EdgeStyle {
    pub stroke: &'static str,
    pub stroke_width: f64,
    pub dash: &'static str,
    pub marker_end: &'static str,
    pub marker_start: &'static str,
    pub semantic_label: &'static str,
}

/// Return the `EdgeStyle` for a given edge kind string.
pub fn edge_style(kind: &str) -> EdgeStyle {
    match kind {
        "flow" => EdgeStyle {
            stroke: "#3a6ea5", stroke_width: 1.4, dash: "",
            marker_end: "arr-flow", marker_start: "", semantic_label: "",
        },
        "derive" | "derivedFrom" => EdgeStyle {
            stroke: "#555", stroke_width: 1.2, dash: "5,3",
            marker_end: "arr-open", marker_start: "", semantic_label: "derive",
        },
        "verify" | "verifies" => EdgeStyle {
            stroke: "#3a6ea5", stroke_width: 1.2, dash: "5,3",
            marker_end: "arr-verify", marker_start: "", semantic_label: "verify",
        },
        "allocate" | "allocatedTo" => EdgeStyle {
            stroke: "#7a3ea5", stroke_width: 1.2, dash: "3,3",
            marker_end: "arr-alloc", marker_start: "", semantic_label: "allocate",
        },
        "satisfy" | "satisfies" => EdgeStyle {
            stroke: "#1a6b30", stroke_width: 1.2, dash: "4,2",
            marker_end: "arr-satisfy", marker_start: "", semantic_label: "satisfy",
        },
        "generalize" | "supertype" => EdgeStyle {
            stroke: "#3a3a5c", stroke_width: 1.4, dash: "",
            marker_end: "arr-generalize", marker_start: "", semantic_label: "",
        },
        "use" => EdgeStyle {
            stroke: "#888", stroke_width: 1.2, dash: "6,3",
            marker_end: "arr-open", marker_start: "", semantic_label: "use",
        },
        "realization" | "realize" => EdgeStyle {
            stroke: "#3a3a5c", stroke_width: 1.2, dash: "5,3",
            marker_end: "arr-generalize", marker_start: "", semantic_label: "",
        },
        "association" => EdgeStyle {
            stroke: "#555", stroke_width: 1.4, dash: "",
            marker_end: "", marker_start: "", semantic_label: "",
        },
        "directed_association" => EdgeStyle {
            stroke: "#555", stroke_width: 1.4, dash: "",
            marker_end: "arr-open", marker_start: "", semantic_label: "",
        },
        "composition" => EdgeStyle {
            stroke: "#555", stroke_width: 1.4, dash: "",
            marker_end: "arr-open", marker_start: "arr-diamond-filled", semantic_label: "",
        },
        "aggregation" => EdgeStyle {
            stroke: "#555", stroke_width: 1.4, dash: "",
            marker_end: "arr-open", marker_start: "arr-diamond-hollow", semantic_label: "",
        },
        "binding" => EdgeStyle {
            stroke: "#555", stroke_width: 1.4, dash: "",
            marker_end: "", marker_start: "", semantic_label: "=",
        },
        "interface_provided" | "provided" => EdgeStyle {
            stroke: "#1a6b30", stroke_width: 1.4, dash: "",
            marker_end: "arr-circle", marker_start: "", semantic_label: "",
        },
        "interface_required" | "required" => EdgeStyle {
            stroke: "#1a6b30", stroke_width: 1.4, dash: "",
            marker_end: "arr-open", marker_start: "", semantic_label: "",
        },
        "include" => EdgeStyle {
            stroke: "#555", stroke_width: 1.2, dash: "5,3",
            marker_end: "arr-open", marker_start: "", semantic_label: "include",
        },
        "extend" => EdgeStyle {
            stroke: "#555", stroke_width: 1.2, dash: "5,3",
            marker_end: "arr-open", marker_start: "", semantic_label: "extend",
        },
        "succession" => EdgeStyle {
            stroke: "#2a6040", stroke_width: 1.4, dash: "",
            marker_end: "arr-filled", marker_start: "", semantic_label: "",
        },
        "item_flow" | "itemflow" => EdgeStyle {
            stroke: "#3a6ea5", stroke_width: 1.2, dash: "3,3",
            marker_end: "arr-flow", marker_start: "", semantic_label: "",
        },
        _ => EdgeStyle {
            stroke: "#888", stroke_width: 1.2, dash: "",
            marker_end: "arr-open", marker_start: "", semantic_label: "",
        },
    }
}

/// Render pre-computed waypoints as an SVG edge with the given style and optional label.
pub fn route_edge_routed(
    waypoints: &[(f64, f64)],
    style: &EdgeStyle,
    user_label: Option<&str>,
) -> String {
    let d = super::astar::waypoints_to_svg(waypoints);

    let dash_attr = if style.dash.is_empty() {
        String::new()
    } else {
        format!(" stroke-dasharray=\"{}\"", style.dash)
    };
    let marker_end_attr = if style.marker_end.is_empty() {
        String::new()
    } else {
        format!(" marker-end=\"url(#{})\"", style.marker_end)
    };
    let marker_start_attr = if style.marker_start.is_empty() {
        String::new()
    } else {
        format!(" marker-start=\"url(#{})\"", style.marker_start)
    };

    let path_svg = format!(
        "<path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"{}{}{}/>",
        d,
        style.stroke,
        style.stroke_width,
        dash_attr,
        marker_end_attr,
        marker_start_attr,
    );

    // Determine label: user_label wins, else semantic_label
    let label = user_label.unwrap_or(style.semantic_label);
    let label_svg = if label.is_empty() {
        String::new()
    } else {
        let (lx, ly) = longest_segment_mid(waypoints);
        if label == "=" {
            format!(
                "<text x=\"{lx:.1}\" y=\"{ly:.1}\" font-size=\"9\" fill=\"{}\" \
                 text-anchor=\"middle\" font-style=\"italic\">{}</text>",
                style.stroke, label
            )
        } else {
            format!(
                "<text x=\"{lx:.1}\" y=\"{ly:.1}\" font-size=\"9\" fill=\"{}\" \
                 text-anchor=\"middle\" font-style=\"italic\">«{}»</text>",
                style.stroke, label
            )
        }
    };

    format!("{}{}", path_svg, label_svg)
}

fn longest_segment_mid(pts: &[(f64, f64)]) -> (f64, f64) {
    if pts.len() < 2 {
        return pts.first().copied().unwrap_or((0.0, 0.0));
    }
    let mut best_len = -1.0_f64;
    let mut best_mid = (0.0_f64, 0.0_f64);
    for i in 0..pts.len() - 1 {
        let (ax, ay) = pts[i];
        let (bx, by) = pts[i + 1];
        let len = (bx - ax).abs() + (by - ay).abs();
        if len > best_len {
            best_len = len;
            best_mid = ((ax + bx) / 2.0, (ay + by) / 2.0 - 6.0);
        }
    }
    best_mid
}

/// V-H-V routing for tree-style edges (top/bottom anchor connections).
/// Used in REQ diagrams where edges connect bottom of parent to top of child.
pub fn route_edge_vert(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    kind: &str,
) -> String {
    let mid_y = (y1 + y2) / 2.0;
    let waypoints: Vec<(f64, f64)> = if (x1 - x2).abs() < 2.0 {
        vec![(x1, y1), (x2, y2)]
    } else {
        vec![(x1, y1), (x1, mid_y), (x2, mid_y), (x2, y2)]
    };
    let style = edge_style(kind);
    route_edge_routed(&waypoints, &style, None)
}

/// Route an edge between two absolute port anchor points.
/// Returns an SVG `<path>` element string (and optional label).
/// For MVP: 3-segment orthogonal routing (horizontal → vertical → horizontal).
pub fn route_edge(
    x1: f64, y1: f64,
    x2: f64, y2: f64,
    kind: &str,
) -> String {
    let mid_x = (x1 + x2) / 2.0;
    let waypoints: Vec<(f64, f64)> = if (y1 - y2).abs() < 2.0 {
        vec![(x1, y1), (x2, y2)]
    } else {
        vec![(x1, y1), (mid_x, y1), (mid_x, y2), (x2, y2)]
    };
    let style = edge_style(kind);
    route_edge_routed(&waypoints, &style, None)
}

/// SVG `<defs>` block containing all arrowhead markers.
pub fn arrowhead_defs() -> String {
    let mut out = String::from("<defs>");

    // Open arrowhead markers (polyline)
    let open_entries: &[(&str, &str)] = &[
        ("arr-open", "#888"),
        ("arr-flow", "#3a6ea5"),
        ("arr-verify", "#3a6ea5"),
        ("arr-alloc", "#7a3ea5"),
        ("arr-satisfy", "#1a6b30"),
    ];
    for (id, color) in open_entries {
        out.push_str(&format!(
            "<marker id=\"{id}\" markerUnits=\"userSpaceOnUse\" \
             markerWidth=\"8\" markerHeight=\"6\" \
             refX=\"7\" refY=\"3\" orient=\"auto\">\
             <polyline points=\"0,0 7,3 0,6\" fill=\"none\" \
             stroke=\"{c}\" stroke-width=\"1.2\"/></marker>",
            id = id, c = color
        ));
    }

    // Hollow triangle for generalization
    out.push_str(
        "<marker id=\"arr-generalize\" markerUnits=\"userSpaceOnUse\" \
         markerWidth=\"10\" markerHeight=\"8\" \
         refX=\"9\" refY=\"4\" orient=\"auto\">\
         <polygon points=\"0,0 9,4 0,8\" fill=\"white\" \
         stroke=\"#3a3a5c\" stroke-width=\"1.2\"/></marker>"
    );

    // Filled arrowhead (for succession)
    out.push_str(
        "<marker id=\"arr-filled\" markerUnits=\"userSpaceOnUse\" \
         markerWidth=\"8\" markerHeight=\"6\" \
         refX=\"7\" refY=\"3\" orient=\"auto\">\
         <polygon points=\"0,0 7,3 0,6\" fill=\"#555\"/></marker>"
    );

    // Filled diamond (composition) — marker-start, refX=0 puts near vertex at path start
    out.push_str(
        "<marker id=\"arr-diamond-filled\" markerUnits=\"userSpaceOnUse\" \
         markerWidth=\"16\" markerHeight=\"10\" \
         refX=\"0\" refY=\"5\" orient=\"auto\">\
         <polygon points=\"0,5 8,0 16,5 8,10\" fill=\"#555\"/></marker>"
    );

    // Hollow diamond (aggregation)
    out.push_str(
        "<marker id=\"arr-diamond-hollow\" markerUnits=\"userSpaceOnUse\" \
         markerWidth=\"16\" markerHeight=\"10\" \
         refX=\"0\" refY=\"5\" orient=\"auto\">\
         <polygon points=\"0,5 8,0 16,5 8,10\" fill=\"white\" stroke=\"#555\" stroke-width=\"1.2\"/></marker>"
    );

    // Circle (interface provided)
    out.push_str(
        "<marker id=\"arr-circle\" markerUnits=\"userSpaceOnUse\" \
         markerWidth=\"12\" markerHeight=\"12\" \
         refX=\"6\" refY=\"6\" orient=\"auto\">\
         <circle cx=\"6\" cy=\"6\" r=\"5\" fill=\"none\" stroke=\"#1a6b30\" stroke-width=\"1.4\"/></marker>"
    );

    out.push_str("</defs>");
    out
}
