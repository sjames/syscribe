//! ELK layout in-process (`REQ-TRS-VIS-016`, `ADR-SYS-VIS-001` addendum).
//!
//! The browser lays diagrams out with `elkjs`; this module runs the very same
//! vendored `elk.bundled.js` (`crates/syscribe-model/vendor/elkjs/`,
//! EPL-2.0, embedded with `include_str!`) inside the executable under
//! QuickJS (`rquickjs`), with no Node runtime, browser or network. The ELK
//! input is built exactly as the client's `SyscribeLayoutConfigurator` and
//! `sprotty-elk`'s transformer build it (`frontend/src/layout.ts`): the same
//! option ids and values at graph, node, port and edge level, pins honoured
//! through `elk.position` with ELK's interactive strategies, the `fixed`
//! algorithm when every node is pinned, edges of the reversed kinds flipped
//! before and their sections flipped back after, hierarchical edge sections
//! translated from their ELK `container` into root coordinates, and — in
//! `fixed` mode, where ELK places no labels — the client's own label
//! placement. ELK is deterministic, so the same input yields the same
//! coordinates here and in the browser (`REQ-TRS-VIS-017`).
//!
//! ## Engine lifetime
//!
//! One QuickJS runtime **per thread**, created on first use and kept in a
//! thread-local: a QuickJS runtime and context are not `Send` (without
//! `rquickjs`'s `parallel` feature), so they cannot live in a process-wide
//! static. Loading the 1.5 MB bundle costs a few hundred milliseconds per
//! thread, a layout afterwards tens of milliseconds for a typical diagram;
//! the CLI lays out on one thread, and a test binary pays the load once per
//! test thread. Nothing here is `async`: promise jobs and the `setTimeout`
//! queue the bundle needs are drained synchronously after each
//! `elk.layout()` call.
//!
//! ## Shims
//!
//! The UMD bundle needs `window`/`global` aliases of `globalThis`, a
//! `console`, and `setTimeout`/`clearTimeout`, which are queued and run by
//! the drain loop. `self` is deliberately **not** defined: with it present,
//! ELK's worker module believes it runs inside a Web Worker and exports
//! nothing.

use std::cell::OnceCell;
use std::collections::BTreeMap;

use rquickjs::{Context, Function, Runtime, Value};
use serde_json::{json, Map, Value as Json};

use super::ir::{DiagramGraph, Node, NodeKind, Point};
use super::size::{carried_size, LabelRole, Sizes, MIN_NODE_HEIGHT, MIN_NODE_WIDTH};
use super::sprotty::{port_side, sprotty_type, LayoutOptions, ROOT_ID, TYPE_NODE, TYPE_PORT};

/// The vendored `elk.bundled.js`.
pub const ELK_BUNDLE: &str = include_str!("../../vendor/elkjs/elk.bundled.js");
/// The vendored bundle's elkjs version (`vendor/elkjs/VERSION`, trimmed).
pub fn elk_version() -> &'static str {
    include_str!("../../vendor/elkjs/VERSION").trim()
}

/// The client's spacing and padding defaults (`layout.ts`).
const NODE_NODE_SPACING: &str = "40";
const LAYER_SPACING: &str = "60";
const COMPOUND_PADDING: f64 = 20.0;

#[derive(Debug, thiserror::Error)]
pub enum LayoutError {
    /// The embedded engine could not be created or the bundle not loaded.
    #[error("ELK engine failed to start: {0}")]
    Engine(String),
    /// ELK rejected the graph; the string is the JavaScript error message.
    #[error("ELK layout failed: {0}")]
    Elk(String),
    /// ELK returned something that is not the JSON graph expected.
    #[error("ELK returned an unreadable result: {0}")]
    Output(String),
}

/// An absolute rectangle in root coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Bounds {
    pub fn cx(&self) -> f64 {
        self.x + self.w / 2.0
    }
    pub fn cy(&self) -> f64 {
        self.y + self.h / 2.0
    }
    pub fn right(&self) -> f64 {
        self.x + self.w
    }
    pub fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

/// One edge's route.
#[derive(Debug, Clone, PartialEq)]
pub struct EdgeRoute {
    /// Absolute polyline: start point, bend points, end point.
    pub points: Vec<Point>,
    /// `true` when ELK routed the edge (the ends sit on the shapes' borders);
    /// `false` for the `fixed` algorithm's centre-to-centre straight line,
    /// which a drawer still has to clip to the end shapes.
    pub routed: bool,
}

/// The result of [`layout`]: absolute geometry for everything drawable.
#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    /// The ELK algorithm that ran (`layered`, or `fixed` for a fully pinned graph).
    pub algorithm: String,
    /// Every IR node — blocks, containers, ports, compartments, free labels —
    /// by id, in root coordinates.
    pub nodes: BTreeMap<String, Bounds>,
    /// Every label box by its sprotty id (`<node>-label`, `<node>-stereotype`,
    /// `<node>-banner-<i>`, `<compartment>-line-<i>`, `<port>-label`,
    /// `<edge>-keyword`, `<edge>-label`), in root coordinates.
    pub labels: BTreeMap<String, Bounds>,
    pub edges: BTreeMap<String, EdgeRoute>,
    /// The root's size as ELK reports it (its padding included).
    pub width: f64,
    pub height: f64,
}

// ── the ELK input ─────────────────────────────────────────────────────────

/// The ELK JSON graph built for a [`DiagramGraph`] plus the facts the
/// post-processor needs back.
#[derive(Debug, Clone, PartialEq)]
pub struct ElkInput {
    pub graph: Json,
    /// Edge ids whose `sources`/`targets` were swapped (reversed kinds).
    pub flipped: Vec<String>,
    pub any_pinned: bool,
    pub all_pinned: bool,
}

/// ELK's interactive options, set on the root and on every compound node
/// when some but not all nodes are pinned (`layout.ts` `INTERACTIVE_OPTIONS`).
fn interactive_options(opts: &mut Map<String, Json>) {
    opts.insert("elk.interactive".into(), json!("true"));
    opts.insert("elk.layered.layering.strategy".into(), json!("INTERACTIVE"));
    opts.insert("elk.layered.crossingMinimization.strategy".into(), json!("INTERACTIVE"));
    opts.insert("elk.layered.considerModelOrder.strategy".into(), json!("NODES_AND_EDGES"));
}

/// `layout.ts` `isContainerKind`: the kinds whose labels sit top-left.
fn is_container_kind(kind: NodeKind) -> bool {
    matches!(kind, NodeKind::Boundary | NodeKind::SystemBoundary | NodeKind::Swimlane | NodeKind::Fragment)
}

/// A JavaScript-style number: integers without a fraction, as `${n}` prints.
fn js_num(v: f64) -> String {
    if v.fract() == 0.0 && v.abs() < 1e15 {
        format!("{}", v as i64)
    } else {
        format!("{v}")
    }
}

fn fallback_side(node: &Node, direction: &str) -> &'static str {
    let right = direction == "RIGHT";
    match node.direction {
        Some(super::ir::PortDirection::In) => {
            if right {
                "WEST"
            } else {
                "NORTH"
            }
        }
        _ => {
            if right {
                "EAST"
            } else {
                "SOUTH"
            }
        }
    }
}

fn is_root(graph: &DiagramGraph, node: &Node) -> bool {
    match node.parent.as_deref() {
        None => true,
        Some(p) => graph.node(p).is_none(),
    }
}

struct Builder<'a> {
    graph: &'a DiagramGraph,
    sizes: &'a Sizes,
    direction: &'static str,
    any_pinned: bool,
    all_pinned: bool,
}

impl Builder<'_> {
    fn label_json(l: &super::size::LabelBox, positioned: bool) -> Json {
        let mut m = Map::new();
        m.insert("id".into(), json!(l.id));
        m.insert("text".into(), json!(l.text));
        if positioned {
            m.insert("x".into(), json!(l.x));
            m.insert("y".into(), json!(l.y));
        }
        m.insert("width".into(), json!(l.w));
        m.insert("height".into(), json!(l.h));
        Json::Object(m)
    }

    fn port_json(&self, port: &Node, parent_fixed: bool) -> Json {
        let carried = carried_size(port, self.sizes);
        let mut opts = Map::new();
        // Centre the square on the border (ELK's default puts it just outside).
        opts.insert("elk.port.borderOffset".into(), json!(js_num(-carried.w / 2.0)));
        if let Some(side) = port_side(port) {
            opts.insert("elk.port.side".into(), json!(side.to_ascii_uppercase()));
        } else if parent_fixed {
            opts.insert("elk.port.side".into(), json!(fallback_side(port, self.direction)));
        }
        let sizing = self.sizes.node(&port.id);
        let mut m = Map::new();
        m.insert("id".into(), json!(port.id));
        m.insert("layoutOptions".into(), Json::Object(opts));
        m.insert(
            "labels".into(),
            Json::Array(sizing.map(|s| s.labels.iter().map(|l| Self::label_json(l, false)).collect()).unwrap_or_default()),
        );
        if let Some(p) = port.pin {
            m.insert("x".into(), json!(p.x));
            m.insert("y".into(), json!(p.y));
        }
        m.insert("width".into(), json!(carried.w));
        m.insert("height".into(), json!(carried.h));
        Json::Object(m)
    }

    fn node_json(&self, node: &Node, depth: usize) -> Json {
        let kids: Vec<&Node> = if depth < self.graph.nodes.len() { self.graph.children_of(&node.id).collect() } else { Vec::new() };
        let child_nodes: Vec<&Node> = kids.iter().copied().filter(|c| sprotty_type(c.kind) == TYPE_NODE).collect();
        let ports: Vec<&Node> = kids.iter().copied().filter(|c| c.kind == NodeKind::Port).collect();
        let compound = !child_nodes.is_empty();
        let sizing = self.sizes.node(&node.id);
        // The carried (server) size is authoritative: a leaf is laid out at
        // exactly it (`PORTS MINIMUM_SIZE`, the size as the minimum); a
        // compound node is at least the client's minimum or it.
        let carried = carried_size(node, self.sizes);
        let any_side = ports.iter().any(|p| port_side(p).is_some());
        let container = is_container_kind(node.kind);

        let mut opts = Map::new();
        if compound {
            opts.insert("elk.nodeSize.constraints".into(), json!("NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE"));
            opts.insert(
                "elk.nodeSize.minimum".into(),
                json!(format!("({}, {})", js_num(MIN_NODE_WIDTH.max(carried.w).ceil()), js_num(MIN_NODE_HEIGHT.max(carried.h).ceil()))),
            );
        } else {
            opts.insert("elk.nodeSize.constraints".into(), json!("PORTS MINIMUM_SIZE"));
            opts.insert("elk.nodeSize.minimum".into(), json!(format!("({}, {})", js_num(carried.w), js_num(carried.h))));
        }
        opts.insert(
            "elk.nodeLabels.placement".into(),
            json!(if super::size::glyph_size(node.kind).is_some() {
                // A fork/join bar or decision/merge diamond shows its label beside the glyph (`layout.ts` `isGlyphKind`).
                "[H_RIGHT, V_CENTER, OUTSIDE]"
            } else if container {
                "[H_LEFT, V_TOP, INSIDE]"
            } else if super::shape::is_symbol(node.kind) {
                // A safety symbol's text sits in the middle of its outline (`layout.ts` `isSymbolKind`).
                "[H_CENTER, V_CENTER, INSIDE]"
            } else {
                "[H_CENTER, V_TOP, INSIDE]"
            }),
        );
        opts.insert("elk.nodeLabels.padding".into(), json!("[top=4,left=8,bottom=4,right=8]"));
        // An initial node opens its region and a final one closes it, whatever
        // the cycle breaker makes of the transitions between (`layout.ts`).
        match node.kind {
            NodeKind::Initial => {
                opts.insert("elk.layered.layering.layerConstraint".into(), json!("FIRST"));
            }
            NodeKind::Final => {
                opts.insert("elk.layered.layering.layerConstraint".into(), json!("LAST"));
            }
            _ => {}
        }
        opts.insert("elk.portLabels.placement".into(), json!("OUTSIDE"));
        opts.insert("elk.portConstraints".into(), json!(if any_side { "FIXED_SIDE" } else { "FREE" }));
        if compound {
            // ELK adds the inside label area to this padding itself.
            let p = js_num(COMPOUND_PADDING);
            opts.insert("elk.padding".into(), json!(format!("[top={p},left={p},bottom={p},right={p}]")));
            if self.all_pinned {
                opts.insert("elk.algorithm".into(), json!("fixed"));
            } else if self.any_pinned {
                interactive_options(&mut opts);
            }
        } else if let Some(s) = sizing {
            // sprotty-elk's `transformCompartment` folds a leaf node's
            // compartment boxes (at their `vbox` offsets inside the carried
            // size) into `org.eclipse.elk.padding`: the client sends it, so
            // this does too.
            let (mut top, mut left, mut bottom, mut right) = (0.0, 0.0, 0.0, 0.0);
            for c in &s.compartments {
                left += c.x;
                top += c.y;
                right += carried.w - c.w - c.x;
                bottom += carried.h - c.h - c.y;
            }
            if top != 0.0 || left != 0.0 || bottom != 0.0 || right != 0.0 {
                opts.insert(
                    "org.eclipse.elk.padding".into(),
                    json!(format!("[top={},left={},bottom={},right={}]", js_num(top), js_num(left), js_num(bottom), js_num(right))),
                );
            }
        }
        if let Some(p) = node.pin {
            opts.insert("elk.position".into(), json!(format!("({}, {})", js_num(p.x), js_num(p.y))));
        }

        let mut m = Map::new();
        m.insert("id".into(), json!(node.id));
        m.insert("layoutOptions".into(), Json::Object(opts));
        if compound {
            m.insert("children".into(), Json::Array(child_nodes.iter().map(|c| self.node_json(c, depth + 1)).collect()));
        }
        // sprotty keeps every edge at the root; a node's own edge list is empty.
        m.insert("edges".into(), Json::Array(Vec::new()));
        m.insert(
            "labels".into(),
            Json::Array(sizing.map(|s| s.labels.iter().map(|l| Self::label_json(l, true)).collect()).unwrap_or_default()),
        );
        m.insert("ports".into(), Json::Array(ports.iter().map(|p| self.port_json(p, any_side)).collect()));
        if let Some(p) = node.pin {
            m.insert("x".into(), json!(p.x));
            m.insert("y".into(), json!(p.y));
        }
        // The carried size, stamped on the ELK node (the client's preprocessor).
        m.insert("width".into(), json!(carried.w));
        m.insert("height".into(), json!(carried.h));
        Json::Object(m)
    }
}

/// Build the ELK JSON for `graph` with `sizes`, as the client would.
pub fn elk_input(graph: &DiagramGraph, sizes: &Sizes) -> ElkInput {
    let opts = LayoutOptions::for_graph(graph);
    let node_ids: Vec<&str> = graph.nodes.iter().filter(|n| sprotty_type(n.kind) == TYPE_NODE).map(|n| n.id.as_str()).collect();
    let pinned = |id: &str| graph.node(id).map(|n| n.pin.is_some()).unwrap_or(false);
    let any_pinned = node_ids.iter().any(|id| pinned(id));
    let all_pinned = !node_ids.is_empty() && node_ids.iter().all(|id| pinned(id));

    let mut graph_opts = Map::new();
    graph_opts.insert("elk.algorithm".into(), json!(if all_pinned { "fixed" } else { opts.algorithm }));
    graph_opts.insert("elk.direction".into(), json!(opts.direction));
    // A feature model with several roots is one picture, its trees side by side.
    if graph.kind == crate::vis::ir::DiagramKind::FeatureModel {
        graph_opts.insert("elk.separateConnectedComponents".into(), json!("false"));
    }
    graph_opts.insert("elk.edgeRouting".into(), json!("ORTHOGONAL"));
    graph_opts.insert("elk.spacing.nodeNode".into(), json!(NODE_NODE_SPACING));
    graph_opts.insert("elk.layered.spacing.nodeNodeBetweenLayers".into(), json!(LAYER_SPACING));
    graph_opts.insert("elk.spacing.edgeNode".into(), json!("30"));
    graph_opts.insert("elk.layered.spacing.edgeNodeBetweenLayers".into(), json!("30"));
    graph_opts.insert("elk.spacing.portPort".into(), json!("16"));
    graph_opts.insert("elk.spacing.labelLabel".into(), json!("1"));
    graph_opts.insert("elk.spacing.labelPortHorizontal".into(), json!("4"));
    graph_opts.insert("elk.spacing.labelPortVertical".into(), json!("2"));
    graph_opts.insert("elk.padding".into(), json!("[top=20,left=20,bottom=20,right=20]"));
    if let Some(h) = opts.hierarchy_handling {
        graph_opts.insert("elk.hierarchyHandling".into(), json!(h));
    }
    if let Some(c) = opts.cycle_breaking {
        graph_opts.insert("elk.layered.cycleBreaking.strategy".into(), json!(c));
    }
    if any_pinned && !all_pinned {
        interactive_options(&mut graph_opts);
    }

    let b = Builder { graph, sizes, direction: opts.direction, any_pinned, all_pinned };
    let children: Vec<Json> = graph.nodes.iter().filter(|n| is_root(graph, n) && sprotty_type(n.kind) == TYPE_NODE).map(|n| b.node_json(n, 0)).collect();

    // An edge end must be a node or port of the ELK graph (a root-level free
    // label is drawn but never sent, exactly as sprotty-elk filters).
    let in_elk = |id: &str| -> bool {
        let Some(n) = graph.node(id) else { return false };
        match sprotty_type(n.kind) {
            TYPE_NODE => true,
            TYPE_PORT => n.parent.as_deref().and_then(|p| graph.node(p)).map(|p| sprotty_type(p.kind) == TYPE_NODE).unwrap_or(false),
            _ => false,
        }
    };
    let mut flipped = Vec::new();
    let mut edges = Vec::new();
    for e in &graph.edges {
        if !in_elk(&e.source) || !in_elk(&e.target) {
            continue;
        }
        // Overlay edges (a feature diagram's cross-tree constraints) are drawn
        // after placement and take no part in layout.
        if graph.layout_hints.overlay_kinds.contains(&e.kind) {
            continue;
        }
        let reversed = graph.layout_hints.reversed_kinds.contains(&e.kind);
        let (src, tgt) = if reversed { (&e.target, &e.source) } else { (&e.source, &e.target) };
        if reversed {
            flipped.push(e.id.clone());
        }
        let mut m = Map::new();
        m.insert("id".into(), json!(e.id));
        m.insert("sources".into(), json!([src]));
        m.insert("targets".into(), json!([tgt]));
        if let Some(labels) = sizes.edges.get(&e.id).filter(|l| !l.is_empty()) {
            m.insert("labels".into(), Json::Array(labels.iter().map(|l| Builder::label_json(l, false)).collect()));
        }
        if let Some(pts) = e.waypoints.as_ref().filter(|p| p.len() >= 2) {
            let pt = |p: &Point| json!({ "x": p.x, "y": p.y });
            m.insert(
                "sections".into(),
                json!([{
                    "id": format!("{}:section", e.id),
                    "startPoint": pt(&pts[0]),
                    "bendPoints": pts[1..pts.len() - 1].iter().map(pt).collect::<Vec<_>>(),
                    "endPoint": pt(&pts[pts.len() - 1]),
                }]),
            );
        }
        edges.push(Json::Object(m));
    }

    let mut root = Map::new();
    root.insert("id".into(), json!(ROOT_ID));
    root.insert("layoutOptions".into(), Json::Object(graph_opts));
    root.insert("children".into(), Json::Array(children));
    root.insert("edges".into(), Json::Array(edges));
    ElkInput { graph: Json::Object(root), flipped, any_pinned, all_pinned }
}

// ── the engine ────────────────────────────────────────────────────────────

const SHIMS: &str = r#"
var window = globalThis; var global = globalThis;
var console = { log: function(){}, warn: function(){}, error: function(){}, info: function(){}, debug: function(){} };
var __timers = [];
function setTimeout(f, ms) { __timers.push(f); return __timers.length; }
function clearTimeout(id) { __timers[id - 1] = null; }
function __runTimers() {
  var ran = false;
  for (var i = 0; i < __timers.length; i++) {
    var f = __timers[i];
    if (f) { __timers[i] = null; ran = true; f(); }
  }
  if (!ran) { __timers = []; }
  return ran;
}
"#;

const HARNESS: &str = r#"
var __elk = new ELK();
var __out = null, __err = null;
function __layout(input) {
  __out = null; __err = null;
  try {
    __elk.layout(JSON.parse(input)).then(
      function (g) { __out = JSON.stringify(g); },
      function (e) { __err = String(e && e.message ? e.message : e); });
  } catch (e) { __err = String(e && e.message ? e.message : e); }
}
"#;

struct Engine {
    rt: Runtime,
    ctx: Context,
}

fn js_error(ctx: &rquickjs::Ctx<'_>, e: rquickjs::Error) -> String {
    let exc = ctx.catch();
    match exc.as_exception() {
        Some(x) => x.message().unwrap_or_else(|| e.to_string()),
        None => e.to_string(),
    }
}

impl Engine {
    fn new() -> Result<Engine, LayoutError> {
        let rt = Runtime::new().map_err(|e| LayoutError::Engine(e.to_string()))?;
        // ELK's layered algorithm recurses; give the interpreter a generous
        // stack guard (the native thread stack is the real bound).
        rt.set_max_stack_size(1 << 20);
        let ctx = Context::full(&rt).map_err(|e| LayoutError::Engine(e.to_string()))?;
        ctx.with(|ctx| {
            for (what, src) in [("shims", SHIMS), ("elk.bundled.js", ELK_BUNDLE), ("harness", HARNESS)] {
                ctx.eval::<(), _>(src).map_err(|e| LayoutError::Engine(format!("{what}: {}", js_error(&ctx, e))))?;
            }
            Ok::<(), LayoutError>(())
        })?;
        Ok(Engine { rt, ctx })
    }

    fn run(&self, input: String) -> Result<String, LayoutError> {
        self.ctx.with(|ctx| {
            let f: Function = ctx.globals().get("__layout").map_err(|e| LayoutError::Engine(e.to_string()))?;
            f.call::<_, ()>((input,)).map_err(|e| LayoutError::Elk(js_error(&ctx, e)))
        })?;
        // Drive the promise job queue and the timer queue to quiescence.
        loop {
            let mut progressed = false;
            loop {
                match self.rt.execute_pending_job() {
                    Ok(true) => progressed = true,
                    Ok(false) => break,
                    Err(e) => return Err(LayoutError::Elk(format!("{e:?}"))),
                }
            }
            let ran = self.ctx.with(|ctx| ctx.eval::<bool, _>("__runTimers()").unwrap_or(false));
            if !progressed && !ran {
                break;
            }
        }
        self.ctx.with(|ctx| {
            let globals = ctx.globals();
            let err: Value = globals.get("__err").map_err(|e| LayoutError::Engine(e.to_string()))?;
            if let Some(s) = err.as_string() {
                return Err(LayoutError::Elk(s.to_string().unwrap_or_default()));
            }
            let out: Value = globals.get("__out").map_err(|e| LayoutError::Engine(e.to_string()))?;
            match out.as_string() {
                Some(s) => s.to_string().map_err(|e| LayoutError::Output(e.to_string())),
                None => Err(LayoutError::Output("layout produced no result".into())),
            }
        })
    }
}

thread_local! {
    /// One engine per thread (module doc): a QuickJS runtime and context are
    /// not `Send`, so each thread that lays out loads its own on first use.
    static ENGINE: OnceCell<Result<Engine, String>> = const { OnceCell::new() };
}

/// Run one ELK JSON graph through the embedded engine and return ELK's
/// output graph. The determinism test feeds a committed input through this
/// and through Node and compares the results.
pub fn run_elk(input: &Json) -> Result<Json, LayoutError> {
    let out = ENGINE.with(|cell| match cell.get_or_init(|| Engine::new().map_err(|e| e.to_string())) {
        Ok(engine) => engine.run(input.to_string()),
        Err(e) => Err(LayoutError::Engine(e.clone())),
    })?;
    serde_json::from_str(&out).map_err(|e| LayoutError::Output(e.to_string()))
}

// ── post-processing ───────────────────────────────────────────────────────

fn num(v: &Json, key: &str) -> f64 {
    v.get(key).and_then(Json::as_f64).unwrap_or(0.0)
}

fn arr<'a>(v: &'a Json, key: &str) -> &'a [Json] {
    v.get(key).and_then(Json::as_array).map(Vec::as_slice).unwrap_or(&[])
}

fn str_of<'a>(v: &'a Json, key: &str) -> Option<&'a str> {
    v.get(key).and_then(Json::as_str)
}

/// Which side of its parent a laid-out port sits on, from its centre
/// (`layout.ts` `sideOf`).
fn side_of(port: &Json, parent_w: f64, parent_h: f64) -> &'static str {
    let cx = num(port, "x") + num(port, "width") / 2.0;
    let cy = num(port, "y") + num(port, "height") / 2.0;
    let mut dist = [("west", cx.abs()), ("east", (cx - parent_w).abs()), ("north", cy.abs()), ("south", (cy - parent_h).abs())];
    dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    dist[0].0
}

struct Post<'a> {
    graph: &'a DiagramGraph,
    sizes: &'a Sizes,
    flipped: &'a [String],
    all_pinned: bool,
    out: Layout,
    /// Absolute origin of every ELK node (the root included), for edge
    /// container translation.
    offsets: BTreeMap<String, (f64, f64)>,
    /// `(abs origin, width, height, compound)` of every ELK node.
    boxes: BTreeMap<String, (f64, f64, f64, f64, bool)>,
}

impl Post<'_> {
    fn walk_node(&mut self, node: &Json, ax: f64, ay: f64, is_root: bool) {
        let id = str_of(node, "id").unwrap_or("").to_string();
        let (x, y) = (ax + num(node, "x"), ay + num(node, "y"));
        let (w, h) = (num(node, "width"), num(node, "height"));
        let children = arr(node, "children");
        self.offsets.insert(id.clone(), (x, y));
        self.boxes.insert(id.clone(), (x, y, w, h, !children.is_empty()));
        if !is_root {
            self.out.nodes.insert(id.clone(), Bounds { x, y, w, h });
            for l in arr(node, "labels") {
                if let Some(lid) = str_of(l, "id") {
                    self.out.labels.insert(lid.to_string(), Bounds { x: x + num(l, "x"), y: y + num(l, "y"), w: num(l, "width"), h: num(l, "height") });
                }
            }
            for p in arr(node, "ports") {
                let Some(pid) = str_of(p, "id") else { continue };
                let (px, py) = (x + num(p, "x"), y + num(p, "y"));
                self.out.nodes.insert(pid.to_string(), Bounds { x: px, y: py, w: num(p, "width"), h: num(p, "height") });
                for l in arr(p, "labels") {
                    if let Some(lid) = str_of(l, "id") {
                        self.out.labels.insert(lid.to_string(), Bounds { x: px + num(l, "x"), y: py + num(l, "y"), w: num(l, "width"), h: num(l, "height") });
                    }
                }
            }
            // Compartments are not ELK elements: they sit at their `vbox`
            // offset and span the node's final width, as the client draws them.
            if let Some(s) = self.sizes.node(&id) {
                for c in &s.compartments {
                    self.out.nodes.insert(c.id.clone(), Bounds { x: x + c.x, y: y + c.y, w, h: c.h });
                    if let Some(cs) = self.sizes.node(&c.id) {
                        for l in &cs.labels {
                            self.out.labels.insert(l.id.clone(), Bounds { x: x + c.x + l.x, y: y + c.y + l.y, w: l.w, h: l.h });
                        }
                    }
                }
            }
        }
        for c in children {
            self.walk_node(c, x, y, false);
        }
    }

    fn walk_edges(&mut self, node: &Json) {
        let owner = str_of(node, "id").unwrap_or("").to_string();
        for e in arr(node, "edges") {
            self.edge(e, &owner);
        }
        for c in arr(node, "children") {
            self.walk_edges(c);
        }
    }

    fn edge(&mut self, e: &Json, owner: &str) {
        let Some(id) = str_of(e, "id") else { return };
        let flipped = self.flipped.iter().any(|f| f == id);
        // ELK reports an edge's sections relative to its `container` (the
        // lowest common ancestor of its ends).
        let container = str_of(e, "container").unwrap_or(owner);
        let (ox, oy) = self.offsets.get(container).copied().unwrap_or((0.0, 0.0));
        let pt = |p: &Json| Point { x: num(p, "x") + ox, y: num(p, "y") + oy };
        let sections = arr(e, "sections");
        let mut points: Vec<Point> = Vec::new();
        if let Some(s) = sections.first() {
            if let Some(sp) = s.get("startPoint") {
                points.push(pt(sp));
            }
            points.extend(arr(s, "bendPoints").iter().map(pt));
            if let Some(ep) = s.get("endPoint") {
                points.push(pt(ep));
            }
            if flipped {
                points.reverse();
            }
        }
        let labels: Vec<(String, Bounds)> = arr(e, "labels")
            .iter()
            .filter_map(|l| {
                let lid = str_of(l, "id")?;
                let placed = l.get("x").is_some() && l.get("y").is_some();
                Some((lid.to_string(), Bounds { x: if placed { num(l, "x") + ox } else { f64::NAN }, y: if placed { num(l, "y") + oy } else { f64::NAN }, w: num(l, "width"), h: num(l, "height") }))
            })
            .collect();
        let routed = !points.is_empty();
        if !routed {
            // `fixed` routes nothing: a straight line between the ends' centres.
            let (src, tgt) = if flipped { (arr(e, "targets"), arr(e, "sources")) } else { (arr(e, "sources"), arr(e, "targets")) };
            let centre = |ids: &[Json]| ids.first().and_then(Json::as_str).and_then(|i| self.out.nodes.get(i)).map(|b| Point { x: b.cx(), y: b.cy() });
            if let (Some(a), Some(b)) = (centre(src), centre(tgt)) {
                points = vec![a, b];
                // The client's `placeLabelsFixed`: stacked above the midpoint.
                let stack: f64 = labels.iter().map(|(_, l)| l.h + 1.0).sum();
                let mut y = (a.y + b.y) / 2.0 - stack - 3.0;
                for (lid, l) in &labels {
                    self.out.labels.insert(lid.clone(), Bounds { x: (a.x + b.x) / 2.0 - l.w / 2.0, y, w: l.w, h: l.h });
                    y += l.h + 1.0;
                }
            }
        } else {
            for (lid, l) in &labels {
                if !l.x.is_nan() {
                    self.out.labels.insert(lid.clone(), *l);
                }
            }
        }
        if !points.is_empty() {
            self.out.edges.insert(id.to_string(), EdgeRoute { points, routed });
        }
    }

    /// The client's `placeLabelsFixed` for nodes and ports: ELK's `fixed`
    /// algorithm leaves every label where it was.
    fn place_labels_fixed(&mut self, node: &Json, is_root: bool) {
        let id = str_of(node, "id").unwrap_or("");
        if !is_root {
            let (x, y, w, _h, compound) = self.boxes.get(id).copied().unwrap_or((0.0, 0.0, 0.0, 0.0, false));
            let mut ly = 4.0;
            for l in arr(node, "labels") {
                let (lw, lh) = (num(l, "width"), num(l, "height"));
                let lx = if compound { 8.0 } else { ((w - lw) / 2.0).max(0.0) };
                if let Some(lid) = str_of(l, "id") {
                    self.out.labels.insert(lid.to_string(), Bounds { x: x + lx, y: y + ly, w: lw, h: lh });
                }
                ly += lh + 1.0;
            }
            let ph_w = num(node, "width");
            let ph_h = num(node, "height");
            for p in arr(node, "ports") {
                let (pw, ph) = (num(p, "width"), num(p, "height"));
                let (px, py) = (x + num(p, "x"), y + num(p, "y"));
                for l in arr(p, "labels") {
                    let (lw, lh) = (num(l, "width"), num(l, "height"));
                    // Below-and-beside for east/west ports, as ELK's own
                    // OUTSIDE placement does, so the label clears the edge line.
                    let (lx, ly) = match side_of(p, ph_w, ph_h) {
                        "west" => (-lw - 1.0, ph + 1.0),
                        "east" => (pw + 1.0, ph + 1.0),
                        "north" => ((pw - lw) / 2.0, -lh - 2.0),
                        _ => ((pw - lw) / 2.0, ph + 2.0),
                    };
                    if let Some(lid) = str_of(l, "id") {
                        self.out.labels.insert(lid.to_string(), Bounds { x: px + lx, y: py + ly, w: lw, h: lh });
                    }
                }
            }
        }
        for c in arr(node, "children") {
            self.place_labels_fixed(c, false);
        }
    }
}

/// Lay `graph` out with the embedded ELK, using `sizes` (from
/// [`super::size::size_graph`]) for every node, port and label.
pub fn layout(graph: &DiagramGraph, sizes: &Sizes) -> Result<Layout, LayoutError> {
    let input = elk_input(graph, sizes);
    let result = run_elk(&input.graph)?;
    Ok(apply(graph, sizes, &input, &result))
}

/// The polyline of a cross-tree constraint between two feature boxes: an S from
/// the lower edge of the upper box to the upper edge of the lower one, or, for
/// two boxes on one level, an arc below them whose depth grows with their
/// distance. The same curve the browser draws (`views.tsx`'s `constraintCurve`),
/// sampled.
pub fn constraint_polyline(a: &Bounds, b: &Bounds) -> Vec<Point> {
    let (ax, bx) = (a.cx(), b.cx());
    let same_level = (a.y - b.y).abs() < a.h.max(b.h);
    let (p0, p1, p2, p3) = if same_level {
        let depth = (34.0 + (bx - ax).abs() * 0.12).min(130.0);
        let (y0, y3) = (a.bottom(), b.bottom());
        (Point { x: ax, y: y0 }, Point { x: ax, y: y0 + depth }, Point { x: bx, y: y3 + depth }, Point { x: bx, y: y3 })
    } else {
        let (hi, lo, flip) = if a.y < b.y { (a, b, false) } else { (b, a, true) };
        let k = ((lo.y - hi.bottom()) / 2.0).max(30.0);
        let start = Point { x: hi.cx(), y: hi.bottom() };
        let end = Point { x: lo.cx(), y: lo.y };
        let q1 = Point { x: start.x, y: start.y + k };
        let q2 = Point { x: end.x, y: end.y - k };
        if flip { (end, q2, q1, start) } else { (start, q1, q2, end) }
    };
    const STEPS: usize = 24;
    (0..=STEPS)
        .map(|i| {
            let t = i as f64 / STEPS as f64;
            let u = 1.0 - t;
            let f = |v0: f64, v1: f64, v2: f64, v3: f64| u * u * u * v0 + 3.0 * u * u * t * v1 + 3.0 * u * t * t * v2 + t * t * t * v3;
            Point { x: f(p0.x, p1.x, p2.x, p3.x), y: f(p0.y, p1.y, p2.y, p3.y) }
        })
        .collect()
}

/// Turn ELK's output for `input` into a [`Layout`] (the client's
/// postprocessor: flip reversed edges back, translate sections and labels
/// into root coordinates, place labels in `fixed` mode).
pub fn apply(graph: &DiagramGraph, sizes: &Sizes, input: &ElkInput, result: &Json) -> Layout {
    let mut post = Post {
        graph,
        sizes,
        flipped: &input.flipped,
        all_pinned: input.all_pinned,
        out: Layout {
            algorithm: if input.all_pinned {
                "fixed".to_string()
            } else {
                match LayoutOptions::for_graph(graph).algorithm {
                    "fixed" => "fixed".to_string(),
                    "mrtree" => "mrtree".to_string(),
                    _ => "layered".to_string(),
                }
            },
            nodes: BTreeMap::new(),
            labels: BTreeMap::new(),
            edges: BTreeMap::new(),
            width: num(result, "width"),
            height: num(result, "height"),
        },
        offsets: BTreeMap::new(),
        boxes: BTreeMap::new(),
    };
    post.walk_node(result, 0.0, 0.0, true);
    if post.all_pinned {
        post.place_labels_fixed(result, true);
    }
    post.walk_edges(result);
    // A feature diagram's edges are drawn by rule, not routed by ELK: a tree edge
    // is a straight line from the parent's bottom centre to the child's top
    // centre; a cross-tree constraint (an overlay edge, not laid out) is a curve
    // that avoids the features between its ends (`REQ-TRS-FMED-001`).
    for e in post.graph.edges.iter() {
        let (Some(s), Some(t)) = (post.out.nodes.get(&e.source), post.out.nodes.get(&e.target)) else { continue };
        if e.kind == crate::vis::ir::EdgeKind::FeatureChild {
            let points = vec![Point { x: s.cx(), y: s.bottom() }, Point { x: t.cx(), y: t.y }];
            post.out.edges.insert(e.id.clone(), EdgeRoute { points, routed: true });
        } else if post.graph.layout_hints.overlay_kinds.contains(&e.kind) {
            post.out.edges.insert(e.id.clone(), EdgeRoute { points: constraint_polyline(s, t), routed: true });
        }
    }
    // Free labels at the root are not ELK elements: stack them beneath the
    // drawing so nothing overlaps.
    let mut y = post.out.height;
    for n in post.graph.nodes.iter().filter(|n| n.kind == NodeKind::Label && is_root(post.graph, n)) {
        let s = post.sizes.size_of(&n.id);
        post.out.nodes.insert(n.id.clone(), Bounds { x: COMPOUND_PADDING, y, w: s.w, h: s.h });
        y += s.h + 4.0;
    }
    if y > post.out.height {
        post.out.height = y + COMPOUND_PADDING;
    }
    let _ = LabelRole::Free;
    post.out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vis::ir::{DiagramKind, Edge, EdgeKind, PortDirection, Rect};
    use crate::vis::metrics::ApproxMetrics;
    use crate::vis::size::size_graph;

    fn node(id: &str, kind: NodeKind, parent: Option<&str>) -> Node {
        Node {
            id: id.into(),
            element_ref: format!("Sys::{id}"),
            resolved: true,
            element_type: None,
            kind,
            label: id.to_uppercase(),
            stereotype: Some("part".into()),
            parent: parent.map(str::to_string),
            direction: None,
            side: None,
            lines: vec![],
            is_abstract: false,
            pin: None,
            banners: vec![],
            feature: None,
            mark: None,
        }
    }

    /// boundary `sys` ⊃ blocks `a` (port `a-out`, out) and `b` (port `b-in`,
    /// in); one flow edge a-out → b-in with a label.
    fn ibd() -> DiagramGraph {
        let mut g = DiagramGraph::empty(DiagramKind::Ibd, "D", "D", Some("Sys"));
        g.nodes.push(node("sys", NodeKind::Boundary, None));
        g.nodes.push(node("a", NodeKind::Block, Some("sys")));
        let mut p = node("a-out", NodeKind::Port, Some("a"));
        p.direction = Some(PortDirection::Out);
        g.nodes.push(p);
        g.nodes.push(node("b", NodeKind::Block, Some("sys")));
        let mut q = node("b-in", NodeKind::Port, Some("b"));
        q.direction = Some(PortDirection::In);
        g.nodes.push(q);
        g.edges.push(Edge { id: "e".into(), element_ref: None, source: "a-out".into(), target: "b-in".into(), kind: EdgeKind::Flow, label: Some("power".into()), waypoints: None });
        g
    }

    #[test]
    fn elk_input_mirrors_the_client_configurator() {
        let g = ibd();
        let sizes = size_graph(&g, &ApproxMetrics);
        let input = elk_input(&g, &sizes);
        let j = &input.graph;
        assert_eq!(j["id"], "sysml-diagram");
        let o = &j["layoutOptions"];
        assert_eq!(o["elk.algorithm"], "layered");
        assert_eq!(o["elk.direction"], "RIGHT");
        assert_eq!(o["elk.hierarchyHandling"], "INCLUDE_CHILDREN");
        assert_eq!(o["elk.edgeRouting"], "ORTHOGONAL");
        assert_eq!(o["elk.spacing.nodeNode"], "40");
        assert_eq!(o["elk.padding"], "[top=20,left=20,bottom=20,right=20]");
        assert!(o.get("elk.interactive").is_none(), "nothing pinned, nothing interactive");
        assert!(o.get("elk.portConstraints").is_none(), "the root's port constraints are a per-node option");
        let sys = &j["children"][0];
        assert_eq!(sys["id"], "sys");
        let so = &sys["layoutOptions"];
        assert_eq!(so["elk.padding"], "[top=20,left=20,bottom=20,right=20]");
        assert_eq!(so["elk.nodeLabels.placement"], "[H_LEFT, V_TOP, INSIDE]");
        assert_eq!(so["elk.nodeSize.constraints"], "NODE_LABELS PORTS PORT_LABELS MINIMUM_SIZE");
        assert_eq!(so["elk.nodeSize.minimum"], "(120, 40)", "a compound node's label stack is below the client's minimum");
        assert_eq!(so["elk.portConstraints"], "FREE", "the boundary has no ports");
        assert_eq!(sys["children"].as_array().unwrap().len(), 2);
        assert_eq!(sys["labels"].as_array().unwrap().len(), 2, "stereotype + name");
        assert_eq!(sys["labels"][0]["id"], "sys-stereotype");
        assert_eq!(sys["labels"][1]["id"], "sys-label");
        assert_eq!(sys["labels"][1]["y"], 4.0 + 12.0 + 1.0);
        let a = &sys["children"][0];
        let ao = &a["layoutOptions"];
        assert_eq!(ao["elk.nodeLabels.placement"], "[H_CENTER, V_TOP, INSIDE]");
        assert_eq!(ao["elk.portConstraints"], "FIXED_SIDE", "a port with a derived side fixes the sides");
        assert!(ao.get("elk.padding").is_none());
        // A sized leaf: `PORTS MINIMUM_SIZE` with the carried size, exactly.
        let mw = sizes.size_of("a");
        assert_eq!(ao["elk.nodeSize.constraints"], "PORTS MINIMUM_SIZE");
        assert_eq!(ao["elk.nodeSize.minimum"], format!("({}, {})", js_num(mw.w), js_num(mw.h)));
        assert_eq!(a["width"], mw.w);
        assert_eq!(a["height"], mw.h);
        assert_eq!(mw.h, 44.0, "one east port: 28 + 16");
        let p = &a["ports"][0];
        assert_eq!(p["layoutOptions"]["elk.port.side"], "EAST");
        assert_eq!(p["layoutOptions"]["elk.port.borderOffset"], "-6");
        assert_eq!(p["width"], 12.0);
        assert_eq!(p["labels"][0]["id"], "a-out-label");
        assert!(p["labels"][0].get("x").is_none(), "port labels are not stacked");
        let e = &j["edges"][0];
        assert_eq!(e["sources"], json!(["a-out"]));
        assert_eq!(e["targets"], json!(["b-in"]));
        assert_eq!(e["labels"][0]["id"], "e-label");
        assert!(e.get("sections").is_none());
        assert!(input.flipped.is_empty());
        assert!(!input.any_pinned && !input.all_pinned);
    }

    #[test]
    fn reversed_kinds_are_flipped_and_pins_switch_the_modes() {
        let mut g = DiagramGraph::empty(DiagramKind::Bdd, "D", "D", None);
        g.nodes.push(node("base", NodeKind::Block, None));
        g.nodes.push(node("sub", NodeKind::Block, None));
        let mut c = node("sub-compartment", NodeKind::Compartment, Some("sub"));
        c.lines = vec!["x : Real".into()];
        g.nodes.push(c);
        g.edges.push(Edge { id: "inh".into(), element_ref: None, source: "sub".into(), target: "base".into(), kind: EdgeKind::Inheritance, label: None, waypoints: None });
        let sizes = size_graph(&g, &ApproxMetrics);
        let input = elk_input(&g, &sizes);
        assert_eq!(input.flipped, vec!["inh"]);
        assert_eq!(input.graph["edges"][0]["sources"], json!(["base"]), "flipped for layering");
        assert_eq!(input.graph["edges"][0]["targets"], json!(["sub"]));
        // A leaf with a compartment carries sprotty-elk's folded padding.
        let sub = &input.graph["children"][1];
        let pad = sub["layoutOptions"]["org.eclipse.elk.padding"].as_str().unwrap();
        let comp = &sizes.node("sub").unwrap().compartments[0];
        let sub_size = sizes.size_of("sub");
        assert_eq!(pad, format!("[top={},left=0,bottom={},right={}]", js_num(comp.y), js_num(sub_size.h - comp.h - comp.y), js_num(sub_size.w - comp.w)));
        assert!(input.graph["children"][0]["layoutOptions"].get("org.eclipse.elk.padding").is_none());

        // One pin: interactive on the root, `elk.position` on the node.
        g.nodes[0].pin = Some(Rect { x: 10.0, y: 20.5, w: None, h: None });
        let input = elk_input(&g, &sizes);
        assert!(input.any_pinned && !input.all_pinned);
        assert_eq!(input.graph["layoutOptions"]["elk.interactive"], "true");
        assert_eq!(input.graph["layoutOptions"]["elk.layered.layering.strategy"], "INTERACTIVE");
        assert_eq!(input.graph["children"][0]["layoutOptions"]["elk.position"], "(10, 20.5)");
        assert_eq!(input.graph["children"][0]["x"], 10.0);
        assert!(input.graph["children"][1]["layoutOptions"].get("elk.position").is_none());

        // All pinned: `fixed`, no interactive options; a pinned size is carried.
        g.nodes[1].pin = Some(Rect { x: 0.0, y: 200.0, w: Some(300.0), h: Some(90.5) });
        let input = elk_input(&g, &sizes);
        assert!(input.all_pinned);
        assert_eq!(input.graph["layoutOptions"]["elk.algorithm"], "fixed");
        assert!(input.graph["layoutOptions"].get("elk.interactive").is_none());
        assert_eq!(input.graph["children"][1]["width"], 300.0);
        assert_eq!(input.graph["children"][1]["height"], 90.5);
        assert_eq!(input.graph["children"][1]["layoutOptions"]["elk.nodeSize.minimum"], "(300, 90.5)");
    }

    #[test]
    fn js_numbers_print_like_javascript() {
        assert_eq!(js_num(40.0), "40");
        assert_eq!(js_num(-6.0), "-6");
        assert_eq!(js_num(20.5), "20.5");
        assert_eq!(js_num(0.0), "0");
    }

    #[test]
    fn the_engine_lays_out_and_reports_elk_errors() {
        let g = ibd();
        let sizes = size_graph(&g, &ApproxMetrics);
        let l = layout(&g, &sizes).expect("layout");
        assert_eq!(l.algorithm, "layered");
        assert!(l.width > 0.0 && l.height > 0.0);
        for id in ["sys", "a", "b", "a-out", "b-in"] {
            assert!(l.nodes.contains_key(id), "{id} placed");
        }
        let sys = l.nodes["sys"];
        let a = l.nodes["a"];
        let b = l.nodes["b"];
        assert!(a.x >= sys.x && a.right() <= sys.right() && b.x >= sys.x && b.right() <= sys.right());
        assert!(a.right() <= b.x || b.right() <= a.x, "siblings do not overlap");
        assert!(l.labels.contains_key("a-label") && l.labels.contains_key("a-out-label") && l.labels.contains_key("e-label"));
        let e = &l.edges["e"];
        assert!(e.routed && e.points.len() >= 2);
        // An ELK error surfaces as `LayoutError::Elk` with ELK's message.
        let bad = json!({ "id": "root", "layoutOptions": { "elk.algorithm": "no.such.algorithm" }, "children": [{ "id": "n", "width": 10, "height": 10 }] });
        match run_elk(&bad) {
            Err(LayoutError::Elk(msg)) => assert!(!msg.is_empty()),
            other => panic!("expected an ELK error, got {other:?}"),
        }
        // The engine survives the error.
        assert!(layout(&g, &sizes).is_ok());
    }
}
