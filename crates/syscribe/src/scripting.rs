//! Rhai extension scripts (`scripts` command family) — REQ-TRS-SCRIPT-001..006.
//!
//! User-authored `.rhai` scripts live in a configured directory **outside** the
//! model (`[scripts] path`, default `.syscribe/scripts/`). They are *tooling*,
//! never model content: the built-in `validate` pass never sees them
//! (REQ-TRS-SCRIPT-001/006). Scripts run in a **sandboxed**, resource-limited,
//! deterministic Rhai engine — no filesystem/network/clock/random/env, `eval`
//! disabled, the module resolver confined to the scripts directory
//! (REQ-TRS-SCRIPT-002). They register either a **command** (`scripts run`) or a
//! **check** (`scripts validate`) (REQ-TRS-SCRIPT-004), and read the model
//! through a read-only API (REQ-TRS-SCRIPT-003).

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use rhai::{Dynamic, Engine, FnPtr, Map, AST};

use syscribe_model::config::ValidateConfig;
use syscribe_model::element::RawElement;
use syscribe_model::validator::{self, ValidationResult};

// ── Resource limits (REQ-TRS-SCRIPT-002) ──────────────────────────────────────
// Bounded so a runaway script aborts in well under a second on commodity
// hardware, while leaving ample headroom for real model traversals.
const MAX_OPERATIONS: u64 = 50_000_000;
const MAX_CALL_LEVELS: usize = 64;
const MAX_STRING_SIZE: usize = 8 * 1024 * 1024;
const MAX_ARRAY_SIZE: usize = 1_000_000;
const MAX_MAP_SIZE: usize = 1_000_000;

// ── Shared host state ─────────────────────────────────────────────────────────

/// A finding emitted by a check via `finding(elem, code, severity, msg)`
/// (REQ-TRS-SCRIPT-003). `check` and `source` are filled in by the host when the
/// owning check is invoked, so the finding renders as `<check>/<code>`
/// (REQ-TRS-SCRIPT-006).
#[derive(Clone)]
pub struct ScriptFinding {
    pub check: String,
    pub code: String,
    pub severity: String,
    pub message: String,
    pub file: String,
    pub source: String,
}

/// Immutable model snapshot shared (by `Rc`) with every `Element`/`Model` handle.
struct ModelData {
    elements: Vec<RawElement>,
    /// Per-element frontmatter as JSON (null-stripped), for `e.field(...)`.
    frontmatter: Vec<serde_json::Value>,
    result: ValidationResult,
}

impl ModelData {
    fn index_key(&self, i: usize) -> &str {
        let e = &self.elements[i];
        e.frontmatter.id.as_deref().unwrap_or(e.qualified_name.as_str())
    }
}

/// The `model` value handed to a command/check function.
#[derive(Clone)]
pub struct Model {
    data: Rc<ModelData>,
}

/// A read-only element handle (REQ-TRS-SCRIPT-003). Holds a shared snapshot plus
/// an index into it, so it is cheap to `Clone` (Rhai requires `Clone`).
#[derive(Clone)]
pub struct Element {
    data: Rc<ModelData>,
    idx: usize,
}

// ── serde_json → Rhai Dynamic ─────────────────────────────────────────────────

/// Map a frontmatter value (already JSON) to a Rhai `Dynamic`. `null`/absent
/// becomes unit `()`. Numbers map to int/float; arrays/objects map to Rhai
/// arrays/maps (REQ-TRS-SCRIPT-003).
fn json_to_dynamic(v: &serde_json::Value) -> Dynamic {
    match v {
        serde_json::Value::Null => Dynamic::UNIT,
        serde_json::Value::Bool(b) => Dynamic::from(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::from(n.to_string())
            }
        }
        serde_json::Value::String(s) => Dynamic::from(s.clone()),
        serde_json::Value::Array(arr) => {
            let a: rhai::Array = arr.iter().map(json_to_dynamic).collect();
            Dynamic::from(a)
        }
        serde_json::Value::Object(obj) => {
            let mut m = Map::new();
            for (k, val) in obj {
                m.insert(k.as_str().into(), json_to_dynamic(val));
            }
            Dynamic::from_map(m)
        }
    }
}

/// A Rhai array of strings.
fn strings_to_array<I: IntoIterator<Item = String>>(it: I) -> Dynamic {
    let a: rhai::Array = it.into_iter().map(Dynamic::from).collect();
    Dynamic::from(a)
}

// ── Registry ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Command,
    Check,
}

impl Kind {
    fn as_str(&self) -> &'static str {
        match self {
            Kind::Command => "command",
            Kind::Check => "check",
        }
    }
}

/// One registered command or check (REQ-TRS-SCRIPT-004).
struct Registration {
    name: String,
    description: String,
    kind: Kind,
    func: FnPtr,
    /// Index into the loaded ASTs — the AST the `FnPtr` was compiled in, needed to
    /// invoke it later (`fnptr.call(&engine, &ast, args)`).
    ast_index: usize,
    source: String,
}

#[derive(Default)]
struct Registry {
    regs: Vec<Registration>,
}

/// The fully loaded script environment: a sandboxed engine, the per-file ASTs,
/// and the registry of commands/checks.
pub struct ScriptEnv {
    engine: Engine,
    asts: Vec<AST>,
    registry: Registry,
}

// ── Loading ───────────────────────────────────────────────────────────────────

/// Recursively discover `*.rhai` files under `dir`, sorted for deterministic
/// load order (REQ-TRS-SCRIPT-001/002).
fn discover_scripts(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in walkdir::WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.is_file() && p.extension().map(|e| e == "rhai").unwrap_or(false) {
            out.push(p.to_path_buf());
        }
    }
    out.sort();
    out
}

/// Build the sandboxed engine (REQ-TRS-SCRIPT-002). No filesystem/network/clock/
/// random/env functions are registered; `eval` is disabled; the module resolver
/// is confined to the scripts directory; resource limits are set.
fn build_engine(
    scripts_dir: &Path,
    findings: Rc<RefCell<Vec<ScriptFinding>>>,
) -> Engine {
    let mut engine = Engine::new();

    // Resource limits.
    engine.set_max_operations(MAX_OPERATIONS);
    engine.set_max_call_levels(MAX_CALL_LEVELS);
    engine.set_max_string_size(MAX_STRING_SIZE);
    engine.set_max_array_size(MAX_ARRAY_SIZE);
    engine.set_max_map_size(MAX_MAP_SIZE);

    // Disable dynamic code evaluation.
    engine.disable_symbol("eval");

    // Module resolver confined to the scripts directory; imports that escape it
    // (`..`, absolute) cannot resolve a file outside the dir.
    let resolver =
        rhai::module_resolvers::FileModuleResolver::new_with_path(scripts_dir.to_path_buf());
    engine.set_module_resolver(resolver);

    // Output: print → stdout, eprint/debug → stderr (the only side effect).
    engine.on_print(|s| println!("{s}"));
    engine.on_debug(|s, _src, _pos| eprintln!("{s}"));
    engine.register_fn("eprint", |s: &str| eprintln!("{s}"));
    // Allow eprint of any printable value (mirrors print).
    engine.register_fn("eprint", |v: Dynamic| eprintln!("{v}"));

    register_model_api(&mut engine);
    register_finding_fn(&mut engine, findings);

    engine
}

/// Register the read-only `Model`/`Element` API (REQ-TRS-SCRIPT-003).
fn register_model_api(engine: &mut Engine) {
    engine.register_type_with_name::<Model>("Model");
    engine.register_type_with_name::<Element>("Element");

    // model.elements()
    engine.register_fn("elements", |m: &mut Model| -> rhai::Array {
        (0..m.data.elements.len())
            .map(|idx| Dynamic::from(Element { data: m.data.clone(), idx }))
            .collect()
    });
    // model.elements_of_type("Type")
    engine.register_fn("elements_of_type", |m: &mut Model, ty: &str| -> rhai::Array {
        (0..m.data.elements.len())
            .filter(|&idx| {
                m.data.elements[idx]
                    .frontmatter
                    .element_type
                    .as_ref()
                    .map(|t| element_type_name(t) == ty)
                    .unwrap_or(false)
            })
            .map(|idx| Dynamic::from(Element { data: m.data.clone(), idx }))
            .collect()
    });
    // model.find("id-or-qname") → Element or unit ()
    engine.register_fn("find", |m: &mut Model, key: &str| -> Dynamic {
        for idx in 0..m.data.elements.len() {
            let e = &m.data.elements[idx];
            if e.frontmatter.id.as_deref() == Some(key) || e.qualified_name == key {
                return Dynamic::from(Element { data: m.data.clone(), idx });
            }
        }
        Dynamic::UNIT
    });

    // Scalar string getters.
    engine.register_get("qname", |e: &mut Element| e.data.elements[e.idx].qualified_name.clone());
    engine.register_get("id", |e: &mut Element| {
        e.data.elements[e.idx]
            .frontmatter
            .id
            .clone()
            .unwrap_or_default()
    });
    engine.register_get("name", |e: &mut Element| {
        let el = &e.data.elements[e.idx];
        el.frontmatter
            .name
            .clone()
            .unwrap_or_else(|| el.qualified_name.rsplit("::").next().unwrap_or("").to_string())
    });
    engine.register_get("title", |e: &mut Element| {
        // The script API keeps a `title` getter for back-compat, but the underlying
        // label field is now `name` (REQ-TRS-NAME-002).
        e.data.elements[e.idx].frontmatter.name.clone().unwrap_or_default()
    });
    engine.register_get("type", |e: &mut Element| {
        e.data.elements[e.idx]
            .frontmatter
            .element_type
            .as_ref()
            .map(element_type_name)
            .unwrap_or_default()
    });
    engine.register_get("status", |e: &mut Element| {
        e.data.elements[e.idx].frontmatter.status.clone().unwrap_or_default()
    });
    engine.register_get("doc", |e: &mut Element| e.data.elements[e.idx].doc.clone());
    engine.register_get("file", |e: &mut Element| e.data.elements[e.idx].file_path.clone());

    // List getters (arrays of strings).
    engine.register_get("tags", |e: &mut Element| {
        strings_to_array(field_string_list(e, "tags"))
    });
    engine.register_get("supertype", |e: &mut Element| {
        strings_to_array(field_string_list(e, "supertype"))
    });
    engine.register_get("typed_by", |e: &mut Element| {
        strings_to_array(field_string_list(e, "typedBy"))
    });
    engine.register_get("subsets", |e: &mut Element| {
        strings_to_array(field_string_list(e, "subsets"))
    });
    engine.register_get("satisfies", |e: &mut Element| {
        strings_to_array(field_string_list(e, "satisfies"))
    });
    engine.register_get("verifies", |e: &mut Element| {
        strings_to_array(field_string_list(e, "verifies"))
    });
    engine.register_get("derived_from", |e: &mut Element| {
        strings_to_array(field_string_list(e, "derivedFrom"))
    });
    engine.register_get("allocated_to", |e: &mut Element| {
        strings_to_array(field_string_list(e, "allocatedTo"))
    });

    // Computed reverse indices (REQ-TRS-SCRIPT-003).
    engine.register_get("verified_by", |e: &mut Element| {
        strings_to_array(computed_index(e, |r, k| r.verified_by.get(k)))
    });
    engine.register_get("derived_children", |e: &mut Element| {
        strings_to_array(computed_index(e, |r, k| r.derived_children.get(k)))
    });
    engine.register_get("refined_by", |e: &mut Element| {
        strings_to_array(computed_index(e, |r, k| r.refined_by.get(k)))
    });
    engine.register_get("allocated_from", |e: &mut Element| {
        strings_to_array(computed_index(e, |r, k| r.allocated_from.get(k)))
    });

    // e.field("key") → Dynamic (unit () if absent).
    engine.register_fn("field", |e: &mut Element, key: &str| -> Dynamic {
        match e.data.frontmatter[e.idx].get(key) {
            Some(v) => json_to_dynamic(v),
            None => Dynamic::UNIT,
        }
    });

    // e.custom_fields → map.
    engine.register_get("custom_fields", |e: &mut Element| -> Map {
        let mut m = Map::new();
        for (k, v) in &e.data.elements[e.idx].frontmatter.custom_fields {
            let jv = serde_json::to_value(v).unwrap_or(serde_json::Value::Null);
            m.insert(k.as_str().into(), json_to_dynamic(&jv));
        }
        m
    });
}

/// Register `finding(elem, code, severity, message)` capturing the shared sink
/// (REQ-TRS-SCRIPT-003). `check`/`source` are stamped by the host when the check
/// runs.
fn register_finding_fn(engine: &mut Engine, sink: Rc<RefCell<Vec<ScriptFinding>>>) {
    engine.register_fn(
        "finding",
        move |elem: Element, code: &str, severity: &str, message: &str| {
            let file = elem.data.elements[elem.idx].file_path.clone();
            sink.borrow_mut().push(ScriptFinding {
                check: String::new(),
                code: code.to_string(),
                severity: severity.to_string(),
                message: message.to_string(),
                file,
                source: String::new(),
            });
        },
    );
}

/// The string list behind a frontmatter `key`, from the element's JSON view.
fn field_string_list(e: &Element, key: &str) -> Vec<String> {
    match e.data.frontmatter[e.idx].get(key) {
        Some(serde_json::Value::Array(arr)) => {
            arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect()
        }
        Some(serde_json::Value::String(s)) => vec![s.clone()],
        _ => Vec::new(),
    }
}

/// A computed reverse index for the element, keyed by id-else-qname.
fn computed_index(
    e: &Element,
    pick: impl for<'a> Fn(&'a ValidationResult, &str) -> Option<&'a Vec<String>>,
) -> Vec<String> {
    let key = e.data.index_key(e.idx);
    pick(&e.data.result, key).cloned().unwrap_or_default()
}

/// The element-type name as scripts address it (`"Requirement"`, `"PartDef"`, …).
fn element_type_name(t: &syscribe_model::element::ElementType) -> String {
    serde_json::to_value(t)
        .ok()
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .unwrap_or_else(|| format!("{t:?}"))
}

/// Build the JSON frontmatter view for every element (null-stripped), used by
/// `e.field(...)` and the list getters.
fn build_frontmatter_json(elements: &[RawElement]) -> Vec<serde_json::Value> {
    elements
        .iter()
        .map(|e| strip_nulls(serde_json::to_value(&e.frontmatter).unwrap_or(serde_json::Value::Null)))
        .collect()
}

fn strip_nulls(v: serde_json::Value) -> serde_json::Value {
    match v {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .filter(|(_, val)| !val.is_null())
                .map(|(k, val)| (k, strip_nulls(val)))
                .collect(),
        ),
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.into_iter().map(strip_nulls).collect())
        }
        other => other,
    }
}

/// Errors surfaced to the CLI dispatcher. The tool reports them (with the script
/// name where relevant) and exits non-zero; it never crashes (REQ-TRS-SCRIPT-002).
pub enum ScriptError {
    /// A fatal load problem (e.g. duplicate name) — message already formatted.
    Load(String),
    /// A named command/check not found / not the right kind.
    NotFound(String),
    /// A runtime failure during a command/check run.
    Runtime(String),
}

impl ScriptError {
    pub fn message(&self) -> &str {
        match self {
            ScriptError::Load(m) | ScriptError::NotFound(m) | ScriptError::Runtime(m) => m,
        }
    }
}

impl ScriptEnv {
    /// Load every `*.rhai` under the configured scripts dir into a sandboxed
    /// engine, running each file's top level to collect registrations
    /// (REQ-TRS-SCRIPT-001/004). A parse/registration error in one file is
    /// reported (named) to stderr and skipped; it does not abort the others
    /// (REQ-TRS-SCRIPT-002). A duplicate command/check name is a fatal load error.
    pub fn load(config: &ValidateConfig) -> Result<Self, ScriptError> {
        let findings: Rc<RefCell<Vec<ScriptFinding>>> = Rc::new(RefCell::new(Vec::new()));
        let scripts_dir = config
            .scripts_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".syscribe/scripts"));

        let mut engine = build_engine(&scripts_dir, findings);

        // Shared registry the host registration fns write into.
        let registry: Rc<RefCell<Vec<(String, String, Kind, FnPtr)>>> =
            Rc::new(RefCell::new(Vec::new()));
        {
            let reg = registry.clone();
            engine.register_fn(
                "register_command",
                move |name: &str, desc: &str, func: FnPtr| {
                    reg.borrow_mut().push((
                        name.to_string(),
                        desc.to_string(),
                        Kind::Command,
                        func,
                    ));
                },
            );
            let reg = registry.clone();
            engine.register_fn(
                "register_check",
                move |name: &str, desc: &str, func: FnPtr| {
                    reg.borrow_mut().push((
                        name.to_string(),
                        desc.to_string(),
                        Kind::Check,
                        func,
                    ));
                },
            );
        }

        let mut asts: Vec<AST> = Vec::new();
        let mut regs: Vec<Registration> = Vec::new();

        if scripts_dir.is_dir() {
            for path in discover_scripts(&scripts_dir) {
                let source = display_source(&scripts_dir, &path);
                // Compile.
                let ast = match engine.compile_file(path.clone()) {
                    Ok(a) => a,
                    Err(err) => {
                        eprintln!("script '{source}': parse error: {err}");
                        continue;
                    }
                };
                let ast_index = asts.len();
                // Run the top level to collect this file's registrations.
                registry.borrow_mut().clear();
                if let Err(err) = engine.run_ast(&ast) {
                    eprintln!("script '{source}': load error: {err}");
                    // Keep the AST (a partial run may still have registered), but
                    // its registrations are taken from the registry below.
                }
                let collected: Vec<_> = registry.borrow_mut().drain(..).collect();
                for (name, description, kind, func) in collected {
                    regs.push(Registration {
                        name,
                        description,
                        kind,
                        func,
                        ast_index,
                        source: source.clone(),
                    });
                }
                asts.push(ast);
            }
        }

        // Duplicate name across all loaded scripts is a fatal load error.
        for i in 0..regs.len() {
            for j in (i + 1)..regs.len() {
                if regs[i].name == regs[j].name {
                    return Err(ScriptError::Load(format!(
                        "duplicate script name '{}' registered by both '{}' and '{}'",
                        regs[i].name, regs[i].source, regs[j].source
                    )));
                }
            }
        }

        Ok(ScriptEnv {
            engine,
            asts,
            registry: Registry { regs },
        })
    }

    fn find(&self, name: &str) -> Option<&Registration> {
        self.registry.regs.iter().find(|r| r.name == name)
    }

    /// `scripts list` (REQ-TRS-SCRIPT-005).
    pub fn list(&self, json: bool) {
        if json {
            let items: Vec<serde_json::Value> = self
                .registry
                .regs
                .iter()
                .map(|r| {
                    serde_json::json!({
                        "name": r.name,
                        "kind": r.kind.as_str(),
                        "description": r.description,
                        "source": r.source,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&items).unwrap());
            return;
        }
        if self.registry.regs.is_empty() {
            println!("No extension scripts are defined.");
            return;
        }
        println!("| Name | Kind | Description | Source |");
        println!("|---|---|---|---|");
        for r in &self.registry.regs {
            println!(
                "| {} | {} | {} | {} |",
                r.name,
                r.kind.as_str(),
                r.description,
                r.source
            );
        }
    }

    /// Build the `model` value from the parsed elements + validation result.
    fn model_value(&self, elements: &[RawElement], config: &ValidateConfig) -> Model {
        let result = validator::validate_with_config(elements, config);
        let frontmatter = build_frontmatter_json(elements);
        Model {
            data: Rc::new(ModelData {
                elements: elements.to_vec(),
                frontmatter,
                result,
            }),
        }
    }

    /// `scripts run <command>` (REQ-TRS-SCRIPT-005). Returns the command's
    /// returned string, or a `ScriptError`.
    pub fn run_command(
        &self,
        elements: &[RawElement],
        config: &ValidateConfig,
        name: &str,
    ) -> Result<String, ScriptError> {
        let reg = match self.find(name) {
            Some(r) => r,
            None => {
                return Err(ScriptError::NotFound(format!(
                    "unknown command '{name}' (run `scripts list` to see available commands)"
                )))
            }
        };
        if reg.kind == Kind::Check {
            return Err(ScriptError::NotFound(format!(
                "'{name}' is a check, not a command; run it with `scripts validate`"
            )));
        }
        let model = self.model_value(elements, config);
        let model_dyn = Dynamic::from(model);
        let ast = &self.asts[reg.ast_index];
        match reg.func.call::<Dynamic>(&self.engine, ast, (model_dyn,)) {
            Ok(v) => {
                if v.is_unit() {
                    Ok(String::new())
                } else {
                    Ok(v.to_string())
                }
            }
            Err(err) => Err(ScriptError::Runtime(format!(
                "command '{name}' ({}) failed: {err}",
                reg.source
            ))),
        }
    }

    /// Run every registered check, returning the collected findings
    /// (REQ-TRS-SCRIPT-006). A check that errors at runtime is reported (named)
    /// but does not mask other checks' findings; `runtime_error` is set so the
    /// caller can exit non-zero.
    pub fn run_checks(
        &self,
        elements: &[RawElement],
        config: &ValidateConfig,
    ) -> (Vec<ScriptFinding>, bool) {
        // A fresh sink for this pass; we rebuild a finding-aware engine so the
        // sink is the one captured by `finding(...)`.
        let sink: Rc<RefCell<Vec<ScriptFinding>>> = Rc::new(RefCell::new(Vec::new()));
        let scripts_dir = config
            .scripts_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from(".syscribe/scripts"));
        let engine = build_engine(&scripts_dir, sink.clone());

        let model = self.model_value(elements, config);
        let mut out: Vec<ScriptFinding> = Vec::new();
        let mut runtime_error = false;

        for reg in self.registry.regs.iter().filter(|r| r.kind == Kind::Check) {
            sink.borrow_mut().clear();
            let model_dyn = Dynamic::from(model.clone());
            let ast = &self.asts[reg.ast_index];
            match reg.func.call::<Dynamic>(&engine, ast, (model_dyn,)) {
                Ok(_) => {}
                Err(err) => {
                    eprintln!("check '{}' ({}) failed: {err}", reg.name, reg.source);
                    runtime_error = true;
                }
            }
            for mut f in sink.borrow_mut().drain(..) {
                f.check = reg.name.clone();
                f.source = reg.source.clone();
                out.push(f);
            }
        }
        (out, runtime_error)
    }
}

/// Display a script's path relative to the scripts dir (forward-slashed), for
/// stable, root-independent reporting.
fn display_source(scripts_dir: &Path, path: &Path) -> String {
    let rel = path.strip_prefix(scripts_dir).unwrap_or(path);
    rel.components()
        .filter_map(|c| match c {
            std::path::Component::Normal(s) => Some(s.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}
