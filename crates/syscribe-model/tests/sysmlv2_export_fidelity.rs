//! GitHub #204: `export-sysml` emits the kinds and fields it used to skip or drop, writes valid
//! SysML v2 for each, and reports (`// dropped: <field> on <qname>`, counted in the summary) every
//! field it still cannot express. Every export here is also fed to the real parser.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use syscribe_model::element::RawElement;
use syscribe_model::sysmlv2::export::{export_sysml, SysmlExport};
use syscribe_model::walker::walk_model;

fn tempdir() -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!("syscribe-sysmlv2-fidelity-{}-{}", std::process::id(), N.fetch_add(1, Ordering::Relaxed)));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

/// A native model: root package `M`, plus the given `(path, content)` files.
fn native(files: &[(&str, &str)]) -> Vec<RawElement> {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "M/_index.md", "---\ntype: Package\nname: M\n---\n");
    for (p, c) in files {
        write(&r, p, c);
    }
    walk_model(&r).unwrap()
}

/// Parse `text` with the real parser (the export must be valid SysML v2 text) and re-ingest it as a
/// `sysmlSubmodel` package, which must raise no parse warning.
fn assert_parses(text: &str) -> Vec<RawElement> {
    if let Err(e) = sysml_v2_parser::parse(text) {
        panic!("export does not parse: {e:?}\n{text}");
    }
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Imp/_index.md", "---\ntype: Package\nname: Imp\nsysmlSubmodel: true\n---\n");
    write(&r, "Imp/out.sysml", text);
    let els = walk_model(&r).unwrap();
    let bad: Vec<_> = els.iter().flat_map(|e| e.derive_findings.iter()).filter(|(c, _, _)| c == "W541").collect();
    assert!(bad.is_empty(), "re-ingest parse failure: {bad:?}\n{text}");
    els
}

/// Export a native model and check the text parses.
fn export(files: &[(&str, &str)]) -> SysmlExport {
    let els = native(files);
    let out = export_sysml(&els, None).unwrap();
    assert_parses(&out.text);
    out
}

/// Export a native model and return the elements ingestion reads back from the text.
fn round_trip(files: &[(&str, &str)]) -> Vec<RawElement> {
    let els = native(files);
    let out = export_sysml(&els, None).unwrap();
    assert_parses(&out.text)
}

fn type_of<'a>(els: &'a [RawElement], q: &str) -> &'a str {
    els.iter()
        .find(|e| e.qualified_name == q)
        .unwrap_or_else(|| panic!("no element {q}"))
        .frontmatter
        .element_type
        .as_ref()
        .map_or("?", |t| t.name())
}

fn has(out: &SysmlExport, needle: &str) {
    assert!(out.text.contains(needle), "missing `{needle}` in:\n{}", out.text);
}

fn lacks(out: &SysmlExport, needle: &str) {
    assert!(!out.text.contains(needle), "unexpected `{needle}` in:\n{}", out.text);
}

fn dropped(out: &SysmlExport, field: &str) -> usize {
    out.report.dropped.get(field).copied().unwrap_or(0)
}

// ── previously skipped kinds ───────────────────────────────────────────────

#[test]
fn skipped_kinds_are_now_exported() {
    let out = export(&[
        ("M/Weights.md", "---\ntype: AttributeDef\nname: Weights\nsupertype: ScalarValues::Real\n---\nA mass type.\n"),
        ("M/Color.md", "---\ntype: EnumerationDef\nname: Color\nvalues:\n  - name: red\n  - name: green\n    value: 2\n---\n"),
        ("L/_index.md", "---\ntype: LibraryPackage\nname: L\n---\nLibrary.\n"),
        ("L/Unit.md", "---\ntype: PartDef\nname: Unit\n---\n"),
        ("M/Drive.md", "---\ntype: UseCaseDef\nname: Drive\nsubject: M::Car\nactors: [M::Driver]\nobjectives: [reach]\n---\nDrive it.\n"),
        ("M/Analyse.md", "---\ntype: AnalysisCaseDef\nname: Analyse\nsubject: M::Car\n---\n"),
        ("M/Check.md", "---\ntype: VerificationCaseDef\nname: Check\nsubject: M::Car\nverifies: [M::Spec]\n---\n"),
        ("M/Spec.md", "---\ntype: RequirementDef\nname: Spec\nsubject: M::Car\n---\n"),
        ("M/Car.md", "---\ntype: PartDef\nname: Car\n---\n"),
        ("M/Driver.md", "---\ntype: PartDef\nname: Driver\n---\n"),
        ("M/Lens.md", "---\ntype: ConcernDef\nname: Lens\nsubject: M::Car\nstakeholders: [M::Driver]\n---\n"),
        ("M/Look.md", "---\ntype: ViewpointDef\nname: Look\nstakeholders: [M::Driver]\nconcerns: [M::Lens]\n---\n"),
        ("M/Tab.md", "---\ntype: RenderingDef\nname: Tab\n---\n"),
        ("M/Layout.md", "---\ntype: ViewDef\nname: Layout\nrendering: M::Tab\n---\n"),
        ("M/Overview.md", "---\ntype: View\nname: Overview\ntypedBy: M::Layout\nviewpoint: M::Look\nexpose:\n  - M::Car\n  - M::Driver\n---\n"),
        ("M/Wiring.md", "---\ntype: Allocation\nname: Wiring\nallocatedFrom: [M::Car]\nallocatedTo: [M::Driver]\n---\n"),
        ("M/Spare.md", "---\ntype: IndividualDef\nname: Spare\nsupertype: M::Car\n---\n"),
    ]);
    assert!(out.report.skipped.is_empty(), "{:?}", out.report.skipped);
    has(&out, "attribute def Weights :> ScalarValues::Real");
    has(&out, "enum def Color {");
    has(&out, "enum red;");
    has(&out, "enum green = 2;");
    has(&out, "library package L {");
    has(&out, "use case def Drive {");
    has(&out, "subject : M::Car;");
    has(&out, "actor driver : M::Driver;");
    has(&out, "objective reach;");
    has(&out, "analysis def Analyse {");
    has(&out, "verification def Check {");
    has(&out, "verify M::Spec;");
    has(&out, "concern def Lens {");
    has(&out, "stakeholder driver : M::Driver;");
    has(&out, "viewpoint def Look {");
    has(&out, "frame concern lens : M::Lens;");
    has(&out, "rendering def Tab;");
    has(&out, "view def Layout {");
    has(&out, "render M::Tab;");
    has(&out, "view Overview : M::Layout {");
    has(&out, "satisfy M::Look;");
    has(&out, "expose M::Car;");
    has(&out, "allocation Wiring allocate M::Car to M::Driver;");
    has(&out, "individual def Spare :> M::Car;");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

// ── fields that used to be dropped silently ────────────────────────────────

#[test]
fn package_imports_and_aliases() {
    let out = export(&[(
        "M/_index.md",
        "---\ntype: Package\nname: M\nimports:\n  - ScalarValues::*\n  - target: ISQ::**\n    isPublic: true\n  - target: Other::Thing\n    filter: isAbstract\naliases:\n  - name: Wheelset\n    for: M::Car\n  - name: Hidden\n    for: M::Car\n    visibility: private\n---\n",
    )]);
    has(&out, "import ScalarValues::*;");
    has(&out, "public import ISQ::**;");
    has(&out, "import Other::Thing;");
    has(&out, "alias Wheelset for M::Car;");
    has(&out, "private alias Hidden for M::Car;");
    // The filter has no text form here: reported, not lost.
    has(&out, "// dropped: imports[2].filter on M");
    assert_eq!(dropped(&out, "imports[2].filter"), 1);
}

#[test]
fn interface_and_connection_ends() {
    let out = export(&[
        ("M/Port.md", "---\ntype: PortDef\nname: Port\n---\n"),
        (
            "M/Link.md",
            "---\ntype: InterfaceDef\nname: Link\nends:\n  - name: a\n    typedBy: M::Port\n  - name: b\n    typedBy: M::Port\n    isConjugated: true\n    multiplicity: \"0..1\"\n---\n",
        ),
        (
            "M/Junction.md",
            "---\ntype: ConnectionDef\nname: Junction\nends:\n  - name: supply\n    typedBy: M::Port\n  - name: ret\n    typedBy: M::Port\n  - name: drain\n    typedBy: M::Port\n    multiplicity: \"0..1\"\n---\n",
        ),
        (
            "M/Rig.md",
            "---\ntype: PartDef\nname: Rig\nconnections:\n  - name: tri\n    typedBy: M::Junction\n    ends:\n      - end: supply\n        binds: pump.flowOut\n      - end: ret\n        binds: tank.flowIn\n      - end: drain\n        binds: sump.drain\n  - typedBy: M::Link\n    from: M::Rig::a.p\n    to: b.p\n---\n",
        ),
    ]);
    has(&out, "interface def Link {");
    has(&out, "end a : M::Port;");
    has(&out, "end b : ~M::Port [0..1];");
    has(&out, "end drain : M::Port [0..1];");
    has(&out, "connection tri : M::Junction connect (supply ::> pump.flowOut, ret ::> tank.flowIn, drain ::> sump.drain);");
    // The endpoint is relative to the owner: no qualified prefix, `.` as the chain separator.
    has(&out, "connection : M::Link connect a.p to b.p;");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

#[test]
fn port_conjugation_is_written_with_a_tilde() {
    let out = export(&[
        ("M/Power.md", "---\ntype: PortDef\nname: Power\n---\n"),
        ("M/PowerRx.md", "---\ntype: PortDef\nname: PowerRx\nconjugates: M::Power\n---\n"),
        (
            "M/Box.md",
            "---\ntype: PartDef\nname: Box\nfeatures:\n  - name: out1\n    type: Port\n    typedBy: M::Power\n  - name: in1\n    type: Port\n    typedBy: M::Power\n    isConjugated: true\n  - name: in2\n    type: Port\n    typedBy: M::PowerRx\n---\n",
        ),
        ("M/box1.md", "---\ntype: Port\nname: box1\ntypedBy: M::Power\nisConjugated: true\n---\n"),
    ]);
    has(&out, "port out1 : M::Power;");
    has(&out, "port in1 : ~M::Power;");
    // A usage typed by the conjugating def is the conjugate of the def it conjugates.
    has(&out, "port in2 : ~M::Power;");
    has(&out, "port box1 : ~M::Power;");
    // The conjugating def itself cannot be declared; that is reported.
    has(&out, "// dropped: conjugates on M::PowerRx");
    assert_eq!(dropped(&out, "conjugates"), 1);
}

#[test]
fn flags_redefines_subsets_and_ordering_keywords() {
    let out = export(&[
        ("M/T.md", "---\ntype: PartDef\nname: T\n---\n"),
        (
            "M/Car.md",
            "---\ntype: PartDef\nname: Car\nisAbstract: true\nfeatures:\n  - name: driver\n    type: Part\n    typedBy: M::T\n    isReference: true\n  - name: wheels\n    type: Part\n    typedBy: M::T\n    multiplicity: \"4\"\n    isOrdered: true\n    isNonunique: true\n  - name: spare\n    type: Part\n    typedBy: M::T\n    multiplicity: \"0..1\"\n    subsets: [wheels]\n  - name: mass\n    typedBy: ScalarValues::Real\n    isDerived: true\n  - name: cap\n    typedBy: ScalarValues::Real\n    isConstant: true\n    value: 5\n    valueKind: initial\n  - name: fixed\n    typedBy: ScalarValues::Real\n    redefines: [base]\n    value: 7\n    valueKind: default-bound\n  - name: locked\n    typedBy: ScalarValues::Real\n    isReadonly: true\n---\n",
        ),
        ("M/Sub.md", "---\ntype: PartDef\nname: Sub\nsupertype: M::Car\n---\n"),
        (
            "M/c1.md",
            "---\ntype: Part\nname: c1\ntypedBy: M::Car\nmultiplicity: \"2\"\nisOrdered: true\nsubsets: [other]\nredefines: base\n---\n",
        ),
        ("M/refd.md", "---\ntype: Part\nname: refd\ntypedBy: M::Car\nisReference: true\n---\n"),
        ("M/ro.md", "---\ntype: Attribute\nname: ro\ntypedBy: ScalarValues::Real\nisReadonly: true\nisDerived: true\n---\n"),
    ]);
    has(&out, "abstract part def Car {");
    has(&out, "ref part driver : M::T;");
    has(&out, "part wheels : M::T [4] ordered nonunique;");
    has(&out, "part spare : M::T [0..1] :> wheels;");
    has(&out, "derived attribute mass : ScalarValues::Real;");
    has(&out, "constant attribute cap : ScalarValues::Real := 5;");
    has(&out, "attribute fixed : ScalarValues::Real :>> base default = 7;");
    has(&out, "part c1 : M::Car [2] ordered :> other :>> base;");
    has(&out, "ref part refd : M::Car;");
    has(&out, "derived attribute ro : ScalarValues::Real {");
    // `readonly` is not part of the grammar: reported on the inline feature and on the element.
    has(&out, "// dropped: features[locked].isReadonly on M::Car");
    has(&out, "// dropped: isReadonly on M::ro");
    assert_eq!(dropped(&out, "isReadonly"), 1);
    assert_eq!(dropped(&out, "features[locked].isReadonly"), 1);
}

#[test]
fn binding_and_flow_connections() {
    let out = export(&[(
        "M/Sys.md",
        "---\ntype: PartDef\nname: Sys\nbindingConnections:\n  - left: M::Sys::a.speed\n    right: b.speed\n  - name: tie\n    left: x\n    right: y\nflowConnections:\n  - from: M::Sys::tank.fuelOut\n    to: engine.fuelIn\n    kind: streaming\n    item: M::Fuel\n  - name: cmd\n    from: ctl.cmdOut\n    to: act.cmdIn\n    kind: message\n    item: M::Cmd\n  - from: p.q\n    to: r.s\n    kind: succession\n---\n",
    )]);
    has(&out, "bind a.speed = b.speed;");
    has(&out, "binding tie bind x = y;");
    has(&out, "flow of M::Fuel from tank.fuelOut to engine.fuelIn;");
    has(&out, "message cmd of M::Cmd from ctl.cmdOut to act.cmdIn;");
    // `succession flow` has no text form.
    has(&out, "// dropped: flowConnections[2] (kind succession has no text form) on M::Sys");
}

#[test]
fn performs_and_exhibits_states() {
    let out = export(&[
        ("M/Drive.md", "---\ntype: ActionDef\nname: Drive\n---\n"),
        ("M/Running.md", "---\ntype: StateDef\nname: Running\n---\n"),
        (
            "M/Car.md",
            "---\ntype: PartDef\nname: Car\nperforms:\n  - M::Drive\n  - name: go\n    typedBy: M::Drive\n    multiplicity: \"2\"\n    redefines: [base]\nexhibitsStates:\n  - M::Running\n---\n",
        ),
    ]);
    has(&out, "perform action drive : M::Drive;");
    has(&out, "perform action go : M::Drive [2] :>> base;");
    has(&out, "exhibit state running : M::Running;");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

#[test]
fn time_slices_snapshots_and_dependencies() {
    let out = export(&[
        ("M/Life.md", "---\ntype: PartDef\nname: Life\n---\n"),
        (
            "M/Story.md",
            "---\ntype: OccurrenceDef\nname: Story\ntimeSlices:\n  - name: young\n    typedBy: M::Life\n    isPortion: true\nsnapshots:\n  - name: now\n    typedBy: M::Life\n---\n",
        ),
        ("M/Core.md", "---\ntype: PartDef\nname: Core\ndependsOn: [M::Life, M::Story]\n---\n"),
    ]);
    has(&out, "occurrence def Story {");
    has(&out, "timeslice young : M::Life;");
    has(&out, "snapshot now : M::Life;");
    has(&out, "dependency from M::Core to M::Life, M::Story;");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

#[test]
fn variation_parallel_and_action_parameters() {
    let out = export(&[
        ("M/V6.md", "---\ntype: PartDef\nname: V6\n---\n"),
        ("M/Engine.md", "---\ntype: PartDef\nname: Engine\nisVariation: true\nisAbstract: true\n---\n"),
        ("M/Engine/v6.md", "---\ntype: Part\nname: v6\ntypedBy: M::V6\nisVariant: true\n---\n"),
        ("M/Car.md", "---\ntype: PartDef\nname: Car\n---\n"),
        ("M/Car/motor.md", "---\ntype: Part\nname: motor\ntypedBy: M::Engine\nisVariation: true\n---\n"),
        ("M/Modes.md", "---\ntype: StateDef\nname: Modes\nisParallel: true\n---\n"),
        (
            "M/Move.md",
            "---\ntype: ActionDef\nname: Move\nparameters:\n  - name: speed\n    typedBy: ScalarValues::Real\n    direction: in\n  - name: dist\n    typedBy: ScalarValues::Real\n    direction: out\n  - name: both\n    direction: inout\n    multiplicity: \"0..*\"\n  - name: ok\n    typedBy: ScalarValues::Boolean\n    direction: return\n---\n",
        ),
    ]);
    // A variation is abstract by definition: the grammar takes no `abstract variation`.
    has(&out, "variation part def Engine {");
    lacks(&out, "abstract variation");
    has(&out, "variant part v6 : M::V6;");
    has(&out, "variation part motor : M::Engine");
    has(&out, "state def Modes parallel {");
    has(&out, "in speed : ScalarValues::Real;");
    has(&out, "out dist : ScalarValues::Real;");
    has(&out, "inout both [0..*];");
    // An action def has no `return` parameter: written as `out`, marked.
    has(&out, "out ok : ScalarValues::Boolean; // return parameter");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

#[test]
fn asserted_constraints_and_requirement_members() {
    let out = export(&[
        ("M/Limit.md", "---\ntype: ConstraintDef\nname: Limit\nexpression: \"x < 5\"\nparameters:\n  - name: x\n    typedBy: ScalarValues::Real\n---\n"),
        ("M/Car.md", "---\ntype: PartDef\nname: Car\n---\n"),
        ("M/Driver.md", "---\ntype: PartDef\nname: Driver\n---\n"),
        ("M/Safe.md", "---\ntype: ConcernDef\nname: Safe\n---\n"),
        ("M/ok.md", "---\ntype: Constraint\nname: ok\ntypedBy: M::Limit\nisAsserted: true\n---\n"),
        ("M/never.md", "---\ntype: Constraint\nname: never\ntypedBy: M::Limit\nisAsserted: true\nisNegated: true\n---\n"),
        (
            "M/MassReq.md",
            "---\ntype: RequirementDef\nname: MassReq\nsubject: M::Car\nactors: [M::Driver]\nstakeholders: [M::Driver]\nframedConcerns: [M::Safe]\nparameters:\n  - name: maxMass\n    typedBy: ScalarValues::Real\nrequires:\n  - typedBy: M::Limit\n  - expression: \"mass < 1500\"\n    name: heavy\nassume:\n  - expression: \"cold\"\n---\nThe mass is bounded.\n",
        ),
    ]);
    has(&out, "assert constraint ok : M::Limit;");
    has(&out, "assert not constraint never : M::Limit;");
    has(&out, "requirement def MassReq {");
    has(&out, "subject : M::Car;");
    has(&out, "actor driver : M::Driver;");
    has(&out, "stakeholder driver : M::Driver;");
    has(&out, "frame concern safe : M::Safe;");
    has(&out, "attribute maxMass : ScalarValues::Real;");
    has(&out, "require constraint limit : M::Limit;");
    has(&out, "require constraint heavy { mass < 1500 }");
    has(&out, "assume constraint assume1 { cold }");
    assert_eq!(out.report.dropped_total(), 0, "{:?}", out.report.dropped_fields);
}

// ── values and names ───────────────────────────────────────────────────────

#[test]
fn expression_values_are_not_string_literals() {
    let out = export(&[(
        "M/Pack.md",
        "---\ntype: PartDef\nname: Pack\nfeatures:\n  - name: capacityWh\n    typedBy: ScalarValues::Real\n    value: 100\n  - name: usableWh\n    typedBy: ScalarValues::Real\n    value: capacityWh * 0.5\n  - name: mode\n    typedBy: M::Color\n    value: M::Color::red\n  - name: label\n    typedBy: ScalarValues::String\n    value: hello world\n  - name: vin\n    typedBy: ScalarValues::String\n    value: VIN-1234\n  - name: quoted\n    typedBy: ScalarValues::String\n    value: '\"abc\"'\n---\n",
    )]);
    has(&out, "attribute usableWh : ScalarValues::Real = capacityWh * 0.5;");
    lacks(&out, "\"capacityWh * 0.5\"");
    has(&out, "attribute mode : M::Color = M::Color::red;");
    // Plain text stays a string literal.
    has(&out, "attribute label : ScalarValues::String = \"hello world\";");
    has(&out, "attribute vin : ScalarValues::String = \"VIN-1234\";");
    has(&out, "attribute quoted : ScalarValues::String = \"abc\";");
}

#[test]
fn an_untyped_feature_takes_its_kind_from_the_typing_definition() {
    let out = export(&[
        ("M/Wheel.md", "---\ntype: PartDef\nname: Wheel\n---\n"),
        ("M/Fuel.md", "---\ntype: ItemDef\nname: Fuel\n---\n"),
        ("M/Pt.md", "---\ntype: PortDef\nname: Pt\n---\n"),
        ("M/Run.md", "---\ntype: ActionDef\nname: Run\n---\n"),
        (
            "M/Car.md",
            "---\ntype: PartDef\nname: Car\nfeatures:\n  - name: w\n    typedBy: Wheel\n  - name: f\n    typedBy: M::Fuel\n  - name: p\n    typedBy: Pt\n  - name: r\n    typedBy: Run\n  - name: n\n    typedBy: ScalarValues::Real\n---\n",
        ),
    ]);
    has(&out, "part w : Wheel;");
    has(&out, "item f : M::Fuel;");
    has(&out, "port p : Pt;");
    has(&out, "action r : Run;");
    has(&out, "attribute n : ScalarValues::Real;");
}

#[test]
fn qualified_names_are_not_quoted_whole() {
    let out = export(&[
        ("M/Open.md", "---\ntype: ActionDef\nname: Open\n---\n"),
        ("M/Door.md", "---\ntype: StateDef\nname: Door\nentryAction: M::Open\nsubStates:\n  - name: a\n    entryAction: M::Open\n---\n"),
        ("M/Wire.md", "---\ntype: PartDef\nname: Wire\nconnections:\n  - from: M::Wire::a::p\n    to: b.q\n---\n"),
        ("M/dep.md", "---\ntype: Dependency\nname: dep\nclients: [M::Open]\nsuppliers: [M::Door]\n---\n"),
    ]);
    has(&out, "entry action M::Open;");
    lacks(&out, "'M::Open'");
    has(&out, "connection connect a.p to b.q;");
    lacks(&out, "'a.p'");
    has(&out, "dependency dep from M::Open to M::Door;");
}

#[test]
fn a_connection_is_not_written_twice() {
    // Ingestion yields both a `connections:` entry on the part and a `Connection` element for the
    // same `connect`: the export must write one `connection`.
    let out = export(&[
        ("M/Link.md", "---\ntype: ConnectionDef\nname: Link\n---\n"),
        ("M/Rig.md", "---\ntype: PartDef\nname: Rig\nconnections:\n  - typedBy: M::Link\n    from: M::Rig::a\n    to: M::Rig::b\n---\n"),
        ("M/Rig/feed.md", "---\ntype: Connection\nname: feed\ntypedBy: M::Link\n---\nFeeds it.\n"),
        ("M/Rig/other.md", "---\ntype: Connection\nname: other\n---\n"),
    ]);
    assert_eq!(out.text.matches("connect a to b").count(), 1, "{}", out.text);
    has(&out, "connection feed : M::Link connect a to b {");
    lacks(&out, "connection : M::Link");
    has(&out, "connection other;");
}

// ── reporting ──────────────────────────────────────────────────────────────

#[test]
fn every_unexpressible_field_is_a_dropped_comment_and_is_counted() {
    let out = export(&[
        ("M/Sys.md", "---\ntype: PartDef\nname: Sys\nconjugates: M::Other\nexpose: [M::Sys]\nisIndividual: true\n---\n"),
        ("M/Gate.md", "---\ntype: StateDef\nname: Gate\nrequires:\n  - expression: x\n---\n"),
    ]);
    has(&out, "// dropped: conjugates on M::Sys");
    has(&out, "// dropped: expose on M::Sys");
    has(&out, "// dropped: requires on M::Gate");
    assert_eq!(out.report.dropped_total(), 4, "{:?}", out.report.dropped_fields);
    assert!(out.report.dropped_fields.contains(&("M::Sys".to_string(), "isIndividual".to_string())));
    // The summary states the drop count; "0" is no longer the only number it can show.
    let line = out.report.summary_line();
    assert!(line.contains("dropped 4 field(s)"), "{line}");
    assert!(out.text.contains("dropped 4 field(s)"), "{}", out.text);
    assert!(line.ends_with("behaviour entries degraded to comments: 0"), "{line}");
}

#[test]
fn a_clean_model_reports_no_drops() {
    let out = export(&[("M/Car.md", "---\ntype: PartDef\nname: Car\n---\nA car.\n")]);
    assert_eq!(out.report.dropped_total(), 0);
    assert!(out.report.summary_line().contains("dropped 0 field(s) (none)"));
    lacks(&out, "// dropped:");
}

// ── whole-model round trips ────────────────────────────────────────────────

fn find_model_dirs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if !p.is_dir() || p.file_name().is_some_and(|n| n == "wasm-plugins") {
            continue;
        }
        if p.file_name().is_some_and(|n| n == "model") && p.join("_index.md").exists() {
            out.push(p.clone());
        }
        find_model_dirs(&p, out);
    }
}

/// Every model the repository ships: the demo models and each `examples/**/model`.
fn repo_models() -> Vec<PathBuf> {
    let r = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap();
    let mut v: Vec<PathBuf> = ["model", "model_auto", "model_mg", "model_sil"].iter().map(|d| r.join(d)).filter(|p| p.is_dir()).collect();
    find_model_dirs(&r.join("examples"), &mut v);
    v
}

#[test]
fn every_repository_model_exports_text_the_parser_accepts() {
    for root in repo_models() {
        let els = walk_model(&root).unwrap();
        let out = export_sysml(&els, None).unwrap();
        assert_parses(&out.text);
        // Nothing is dropped without a comment: every counted drop has its `// dropped:` line.
        assert_eq!(
            out.text.matches("// dropped: ").count(),
            out.report.dropped_total(),
            "{}: drop comments and the counted drops disagree",
            root.display()
        );
        assert!(!out.text.contains("connection connect '"), "{}: quoted connect endpoint", root.display());
    }
}

#[test]
fn newly_exported_kinds_are_read_back_as_the_same_kinds() {
    let els = round_trip(&[
        ("M/Weights.md", "---\ntype: AttributeDef\nname: Weights\n---\n"),
        ("M/Color.md", "---\ntype: EnumerationDef\nname: Color\nvalues:\n  - name: red\n  - name: green\n---\n"),
        ("M/Car.md", "---\ntype: PartDef\nname: Car\n---\n"),
        ("M/Driver.md", "---\ntype: PartDef\nname: Driver\n---\n"),
        ("M/Drive.md", "---\ntype: UseCaseDef\nname: Drive\nsubject: M::Car\nactors: [M::Driver]\n---\n"),
        ("M/Analyse.md", "---\ntype: AnalysisCaseDef\nname: Analyse\nsubject: M::Car\n---\n"),
        ("M/Check.md", "---\ntype: VerificationCaseDef\nname: Check\nsubject: M::Car\n---\n"),
        ("M/Lens.md", "---\ntype: ConcernDef\nname: Lens\n---\n"),
        ("M/Look.md", "---\ntype: ViewpointDef\nname: Look\nstakeholders: [M::Driver]\n---\n"),
        ("M/Layout.md", "---\ntype: ViewDef\nname: Layout\n---\n"),
        ("M/Wiring.md", "---\ntype: Allocation\nname: Wiring\nallocatedFrom: [M::Car]\nallocatedTo: [M::Driver]\n---\n"),
        ("M/Spare.md", "---\ntype: IndividualDef\nname: Spare\n---\n"),
    ]);
    for (name, ty) in [
        ("Weights", "AttributeDef"),
        ("Color", "EnumerationDef"),
        ("Drive", "UseCaseDef"),
        ("Analyse", "AnalysisCaseDef"),
        ("Check", "VerificationCaseDef"),
        ("Lens", "ConcernDef"),
        ("Look", "ViewpointDef"),
        ("Layout", "ViewDef"),
        ("Wiring", "Allocation"),
        ("Spare", "IndividualDef"),
    ] {
        assert_eq!(type_of(&els, &format!("Imp::M::{name}")), ty, "{name}");
    }
    let color = els.iter().find(|e| e.qualified_name == "Imp::M::Color").unwrap();
    assert_eq!(color.frontmatter.values.as_ref().map(Vec::len), Some(2));
    let drive = els.iter().find(|e| e.qualified_name == "Imp::M::Drive").unwrap();
    assert_eq!(drive.frontmatter.subject.as_deref(), Some("M::Car"));
    assert_eq!(drive.frontmatter.actors.as_deref(), Some(&["M::Driver".to_string()][..]));
    let wiring = els.iter().find(|e| e.qualified_name == "Imp::M::Wiring").unwrap();
    assert_eq!(wiring.frontmatter.allocated_from.as_deref(), Some(&["Imp::M::Car".to_string()][..]));
}

/// GH #206: a dotted `connect eng.pwr to wheel.inp` is ingested as the full `::` path and
/// exported back as the same dotted chain.
#[test]
fn dotted_connect_endpoints_round_trip() {
    let r = tempdir();
    write(&r, "_index.md", "---\ntype: Package\nname: Root\n---\n");
    write(&r, "Sys/_index.md", "---\ntype: Package\nname: Sys\nsysmlSubmodel: true\n---\n");
    write(
        &r,
        "Sys/m.sysml",
        "package P {\n port def PP;\n part def Eng { port pwr : PP; port inp : PP; }\n part def Car { part eng : Eng; part wheel : Eng; connect eng.pwr to wheel.inp; }\n}\n",
    );
    let els = walk_model(&r).unwrap();
    let car = els.iter().find(|e| e.qualified_name == "Sys::P::Car").unwrap();
    let conns = car.frontmatter.connections.as_ref().expect("connections");
    let c = conns[0].as_mapping().unwrap();
    assert_eq!(c.get("from").and_then(|v| v.as_str()), Some("Sys::P::Car::eng::pwr"));
    assert_eq!(c.get("to").and_then(|v| v.as_str()), Some("Sys::P::Car::wheel::inp"));

    let out = export_sysml(&els, None).unwrap();
    has(&out, "connect eng.pwr to wheel.inp");
    let back = assert_parses(&out.text);
    let car = back.iter().find(|e| e.qualified_name.ends_with("::Car")).expect("Car re-ingested");
    let c = car.frontmatter.connections.as_ref().expect("connections after round trip")[0].as_mapping().unwrap().clone();
    assert!(c.get("from").and_then(|v| v.as_str()).is_some_and(|f| f.ends_with("::Car::eng::pwr")), "{c:?}");
    assert!(c.get("to").and_then(|v| v.as_str()).is_some_and(|f| f.ends_with("::Car::wheel::inp")), "{c:?}");
}
