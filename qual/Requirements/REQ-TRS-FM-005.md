---
id: REQ-TRS-FM-005
type: Requirement
name: Tool shall accept a whole feature model authored as one flat, dot-named featureTree sheet, exploded into ordinary FeatureDef elements
status: draft
reqDomain: software
verificationMethod: test
---

Today a feature model with N features requires N separate `.md` files nested in a directory tree (one `FeatureDef` per file; nesting via directory structure or `parentFeature:`). For a large or fast-iterating feature model this is heavyweight to author and review. This requirement adds a **single-file, additive/opt-in** authoring form: one `.md` file (`type: FeatureModel`) whose frontmatter carries the whole tree as a **flat** `featureTree:` list plus an optional flat `crossTreeConstraints:` list, exploded at load time into the same `FeatureDef` elements the multi-file form would produce — so every existing consumer (validator, `feature-check`, `matrix`, `configure`, the web UI) is unaffected.

The tool **shall**:

- recognize a new element type `FeatureModel`, carrying an optional `featureTree:` field: a **flat** list of entries, each shaped like a `FeatureDef`'s own frontmatter (`id`, `mandatory`, `groupKind`, `cardinality`, `parentFeature`, `contributesTo`, `requires`, `excludes`, `parameters`, `buildExports`) plus an optional prose `doc:` string, but whose `name:` is a **dot-separated path relative to the sheet** (e.g. `Platform.CortexM`) rather than a single basic name;
- explode each `FeatureModel` sheet's `featureTree:`, at load time (before validation), into one synthetic `FeatureDef` `RawElement` per entry — no recursion needed, since the whole tree shape is flat — whose qualified name is the sheet's own qualified name, then every `.`-separated segment of the entry's `name:` joined by `::`, i.e. **exactly** the qname a directory-per-feature layout would produce for the same tree shape; an ancestor path prefix needs no entry of its own (mirrors today's multi-file behavior: a qname prefix that is not itself a `FeatureDef` simply implies no parent);
- rewrite the synthesized `FeatureDef`'s own `name:` to just the **last** path segment (the leaf label a per-file `FeatureDef` would carry), carrying every other recognized field through unchanged, so it is indistinguishable from a hand-authored `FeatureDef` file to every downstream pass;
- additionally accept an optional `crossTreeConstraints:` field on the same sheet: a flat list of `{ feature, requires, excludes }` entries, kept separate from the structural `featureTree:` so the model's requires/excludes edges can be reviewed as one section (inline `requires:`/`excludes:` on a `featureTree:` entry continues to work too — this section is additive, not a replacement); `feature`/`requires`/`excludes` values resolve identically: containing `::` → an already-absolute qname; starting with `FEAT` → a stable id; otherwise → a dot-separated path relative to the sheet, resolved the same way a `featureTree:` entry's `name:` is; each entry's resolved `requires`/`excludes` are merged into the matching synthesized `FeatureDef`'s own field;
- accept `parameterConstraints:` (§9.7 cross-feature numeric constraints) directly on a `type: FeatureModel` sheet, not only on a `type: Package`/`LibraryPackage`/`Namespace` `_index.md` as today — the sheet is the feature model's natural home for it now;
- raise a new error `E231` when a `featureTree:` entry has no `name:`, is not a mapping, or its dotted path has an empty segment (leading, trailing, or doubled `.`) — such an entry cannot be placed in the qname tree and is dropped;
- raise a new error `E232` when two `featureTree:` entries (within one sheet or across sheets in the same model) resolve to the same qualified name;
- raise a new error `E233` when a `crossTreeConstraints:` entry is malformed (not a mapping, no `feature:`, or a reference with an empty path segment), or its `feature:` does not resolve to a `FeatureDef` synthesized from that **same sheet's own** `featureTree:` — there is nothing local to attach the constraint to;
- raise a new warning `W048` when `featureTree:` is declared on an element whose `type:` is not `FeatureModel` (the field is silently inert there otherwise);
- leave the existing multi-file, directory-per-feature authoring form fully supported and unaffected — the two forms may even be mixed within one model (a `FeatureModel` sheet in one package, ordinary per-file `FeatureDef`s in another), since both simply produce the same kind of `FeatureDef` `RawElement`.

**Out of scope:** an analogous single-file form for `Configuration` is **not** needed — a `Configuration` is already exactly one file today (§9.8), and it addresses features purely by qname/id, with no dependency on how the `FeatureDef` was authored; this requirement only removes the *multi-file* burden that is specific to the feature-model tree.

**Source:** §9.6 (FeatureDef), §9.7 (parameters, `parameterConstraints`), §9.8 (Configuration); the FMEA/TARA sheet-explode precedent (`walker::explode_fmea_entries`/`explode_tara_entries`).

**Acceptance criteria:**

- A `FeatureModel` sheet with a flat, dot-named `featureTree:` (a mandatory XOR group `Platform` with two optional children `Platform.CortexM`/`Platform.RiscV`, and a sibling `Wdt`) produces `FeatureDef` elements with the same qnames, `feature-check --deep` results, and `matrix`/`configure` behavior as the equivalent hand-authored multi-file tree.
- A `crossTreeConstraints:` entry `{ feature: Wdt, requires: [Platform.CortexM] }` on that sheet produces the same effective constraint as an inline `requires: [Features::Platform::CortexM]` on the `Wdt` entry.
- A `featureTree:` entry missing `name:` produces exactly one `E231` naming the sheet file; a duplicate-qname entry produces `E232`.
- A `crossTreeConstraints:` entry whose `feature:` does not resolve within the sheet produces `E233`.
- `featureTree:` declared on a `type: Package` (not `FeatureModel`) produces `W048` and contributes no `FeatureDef` elements.
- `parameterConstraints:` declared directly on a `type: FeatureModel` sheet is evaluated by `feature-check` exactly as it would be on a `type: Package` `_index.md`.
- A `Configuration` selecting features from an exploded `FeatureModel` sheet resolves and validates identically to selecting the same features from per-file `FeatureDef`s.
