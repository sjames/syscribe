use std::collections::HashMap;
use syscribe_model::{element::RawElement, resolver::Resolver};

/// Entry point for `syscribe <model_root> render <diagram_path>`.
/// Outputs the rendered diagram to stdout:
///   - Mermaid: the mermaid block content with `click href` directives injected
///   - SVG:     the raw SVG with `<a href="...">` wrappers injected around linked shapes
pub fn cmd_render(elements: &[RawElement], resolver: &Resolver, diagram_path: &str) {
    let elem = elements.iter().find(|e| {
        e.file_path == diagram_path
            || e.file_path.ends_with(&format!("/{}", diagram_path))
    });

    let elem = match elem {
        Some(e) => e,
        None => {
            eprintln!("error: no element found at '{}'", diagram_path);
            std::process::exit(1);
        }
    };

    match elem.frontmatter.diagram_kind.as_deref() {
        Some("Mermaid") => render_mermaid(elements, resolver, elem),
        Some(_) => render_svg(elements, resolver, elem),
        None => {
            eprintln!("error: '{}' has no diagramKind", diagram_path);
            std::process::exit(1);
        }
    }
}

// ── Mermaid ──────────────────────────────────────────────────────────────────

fn render_mermaid(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) {
    let fence = "```mermaid";
    let fence_start = match elem.doc.find(fence) {
        Some(s) => s,
        None => { eprintln!("error: no ```mermaid block in '{}'", elem.file_path); std::process::exit(1); }
    };
    let after_fence = fence_start + fence.len();
    let block_end = match elem.doc[after_fence..].find("```") {
        Some(e) => after_fence + e,
        None => { eprintln!("error: unclosed ```mermaid block in '{}'", elem.file_path); std::process::exit(1); }
    };
    let block = &elem.doc[after_fence..block_end];

    // Collect %% link: NodeId QualifiedName → Mermaid click directives
    let mut click_lines: Vec<String> = Vec::new();
    for line in block.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("%% link:") {
            let rest = rest.trim();
            let mut parts = rest.splitn(2, ' ');
            let node_id = parts.next().unwrap_or("").trim();
            let qn      = parts.next().unwrap_or("").trim();
            if !node_id.is_empty() && !qn.is_empty() {
                if let Some(target) = resolver.resolve_ref(elements, qn) {
                    let url = relative_url(&elem.file_path, &target.file_path);
                    click_lines.push(format!("click {} href \"{}\" _blank", node_id, url));
                }
            }
        }
    }

    print!("{}", block.trim_end_matches('\n'));
    for line in &click_lines {
        println!();
        print!("{}", line);
    }
    println!();
}

// ── SVG ───────────────────────────────────────────────────────────────────────

fn render_svg(elements: &[RawElement], resolver: &Resolver, elem: &RawElement) {
    // Build shape-id → href from shapes frontmatter entries that carry link:
    let mut links: HashMap<String, String> = HashMap::new();
    if let Some(serde_yaml::Value::Mapping(shapes_map)) = &elem.frontmatter.shapes {
        for (k, v) in shapes_map {
            let shape_id = match k.as_str() { Some(s) => s.to_string(), None => continue };
            let attrs = match v { serde_yaml::Value::Mapping(m) => m, _ => continue };

            let link_qn: Option<String> =
                match attrs.get(&serde_yaml::Value::String("link".into())) {
                    Some(serde_yaml::Value::String(s)) if !s.is_empty() => Some(s.clone()),
                    Some(serde_yaml::Value::Bool(true)) => attrs
                        .get(&serde_yaml::Value::String("ref".into()))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    _ => None,
                };

            if let Some(qn) = link_qn {
                if let Some(target) = resolver.resolve_ref(elements, &qn) {
                    links.insert(shape_id, relative_url(&elem.file_path, &target.file_path));
                }
            }
        }
    }

    // Extract the ```svg fenced block
    let fence = "```svg";
    let fence_start = match elem.doc.find(fence) {
        Some(s) => s,
        None => { eprintln!("error: no ```svg block in '{}'", elem.file_path); std::process::exit(1); }
    };
    let after_fence = fence_start + fence.len();
    let block_end = match elem.doc[after_fence..].find("```") {
        Some(e) => after_fence + e,
        None => { eprintln!("error: unclosed ```svg block in '{}'", elem.file_path); std::process::exit(1); }
    };
    let svg = elem.doc[after_fence..block_end].trim();

    let output = if links.is_empty() {
        svg.to_string()
    } else {
        inject_svg_links(svg, &links)
    };
    println!("{}", output);
}

/// Parse `svg` as XML and wrap every `<g id="shape-id">` element (where shape-id
/// is in `links`) with `<a href="url">...</a>`.  All other markup is passed through
/// verbatim so whitespace and attribute ordering are preserved.
fn inject_svg_links(svg: &str, links: &HashMap<String, String>) -> String {
    use quick_xml::{events::Event, Reader, Writer};

    let mut reader = Reader::from_str(svg);
    let mut buf    = Vec::new();
    let mut writer = Writer::new(Vec::new());
    let mut depth: i32 = 0;
    // Stack of depths at which we opened an <a> wrapper (one entry per open wrapper).
    let mut wrap_depths: Vec<i32> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                // Inject <a href="..."> before any <g id="..."> with a matching link.
                if e.name().as_ref() == b"g" {
                    let id = e.attributes()
                        .filter_map(|a| a.ok())
                        .find(|a| a.key.as_ref() == b"id")
                        .and_then(|a| std::str::from_utf8(a.value.as_ref()).ok().map(|s| s.to_string()));
                    if let Some(id_str) = id {
                        if let Some(href) = links.get(&id_str) {
                            writer.get_mut().extend_from_slice(
                                format!("<a href=\"{}\">", href).as_bytes()
                            );
                            wrap_depths.push(depth);
                        }
                    }
                }
                depth += 1;
                writer.write_event(Event::Start(e)).ok();
            }
            Ok(Event::End(e)) => {
                depth -= 1;
                writer.write_event(Event::End(e)).ok();
                // If this closing tag matched a wrapped <g>, close the <a> wrapper.
                if wrap_depths.last() == Some(&depth) {
                    wrap_depths.pop();
                    writer.get_mut().extend_from_slice(b"</a>");
                }
            }
            Ok(Event::Eof) => break,
            Ok(e) => { writer.write_event(e).ok(); }
            Err(e) => {
                eprintln!("SVG parse error: {}", e);
                return svg.to_string();
            }
        }
        buf.clear();
    }

    String::from_utf8(writer.into_inner()).unwrap_or_else(|_| svg.to_string())
}

// ── Shared utilities ──────────────────────────────────────────────────────────

/// Compute the relative URL from `from_file` to `to_file` without touching
/// the filesystem.  Both paths are relative to the same working directory.
fn relative_url(from_file: &str, to_file: &str) -> String {
    use std::path::{Component, Path};

    let from_dir: Vec<_> = Path::new(from_file)
        .parent()
        .unwrap_or(Path::new("."))
        .components()
        .collect();
    let to_parts: Vec<_> = Path::new(to_file).components().collect();

    let common = from_dir.iter().zip(to_parts.iter())
        .take_while(|(a, b)| a == b)
        .count();

    let ups = from_dir.len() - common;
    let mut parts: Vec<String> = (0..ups).map(|_| "..".to_string()).collect();
    for c in &to_parts[common..] {
        if let Component::Normal(s) = c {
            parts.push(s.to_string_lossy().into_owned());
        }
    }
    if parts.is_empty() { to_file.to_string() } else { parts.join("/") }
}
