# PlanningItem Example: Return-to-Home

A small, standalone UAV-flavored model demonstrating every capability of the
native `PlanningItem` element type (`ADR-SYS-PLANITEM-001`,
`REQ-TRS-PLANITEM-000` through `-006`) in one coherent scenario: the
engineering work to add an automatic Return-to-Home-on-low-battery (RTH)
capability. It is a separate model root from this repository's own `model/`
— running validation here never affects that model's baseline.

A second, deliberately-broken model root (`error-demo/model/`) isolates
`REQ-TRS-PLANITEM-006`'s leaf-evidence rule actually *firing*, so the main
example stays a clean, 0-error demonstration.

## Running it

```bash
cargo build --workspace   # once, if you haven't already

# Main example — clean, 0 errors
./target/debug/syscribe -m examples/planning-item/model
./target/debug/syscribe -m examples/planning-item/model feature-check --deep
./target/debug/syscribe -m examples/planning-item/model validate --config CONF-BASE-001
./target/debug/syscribe -m examples/planning-item/model validate --config CONF-PREMIUM-001
./target/debug/syscribe -m examples/planning-item/model why-active Planning::PI-RTH-CLOUDLOG-001 --config CONF-BASE-001
./target/debug/syscribe -m examples/planning-item/model why-active Planning::PI-RTH-CLOUDLOG-001 --config CONF-PREMIUM-001

# Deliberate error demonstration — 2 errors (E719 x2), on purpose
./target/debug/syscribe -m examples/planning-item/error-demo/model
```

Current output: main example is **0 errors, 5 warnings** (all
expected/documented below); `feature-check --deep` is **0 errors, 1
warning** (also documented below), reports `void model: false`, and both
`Configuration`s project as expected. The error-demo model is **2 errors
(E719), 2 warnings**, exactly as intended — see "Deliberate error
demonstration" below.

## Scenario

A small UAV needs automatic Return-to-Home behavior when battery capacity
drops critically low. `Planning::PI-RTH-001` is the top-level `PlanningItem`
tracking that work, broken down into design, implementation, testing,
documentation, a field-bug fix, and a premium-tier cloud-logging feature —
each a child (or grandchild) `PlanningItem`, at a mix of `status` values and
`itemType`s, some already `done` with real evidence, some still `todo` or
`blocked`.

## File layout

```
examples/planning-item/
  README.md                          This file
  model/                             Main example — clean, 0 errors
    .syscribe.toml                   [users] roster: alice, bob (REQ-TRS-PLANITEM-008)
    _index.md                        Root package
    Decisions/
      ADR-RTH-001.md                 breakdownAdr target for REQ-RTH-002/003
    Requirements/
      REQ-RTH-001.md                 top-level goal (achieves target, id form)
      REQ-RTH-002.md                 derived req (achieves target, id form)
      REQ-RTH-003.md                 derived req (achieves target, qname form)
    Tests/
      TC-RTH-BATT-001.md             ref: evidence target (TestCase)
      TC-RTH-NOISE-001.md            ref: evidence target (TestCase)
    Features/
      CloudSync.md                   FeatureDef FEAT-CLOUD-SYNC (standalone optional)
    Configurations/
      CONF-BASE-001.md               selects CloudSync: false
      CONF-PREMIUM-001.md            selects CloudSync: true
    docs/
      rth-design.txt                 real local file — path: evidence target
                                      (no `.md` extension, so the walker never
                                      treats it as a model element)
    Planning/
      PI-RTH-001.md                  top-level, itemType: feature, non-leaf
      PI-RTH-DESIGN-001.md           child, task, leaf, done (path: + waived ref:)
      PI-RTH-IMPL-001.md             child, task, non-leaf (has grandchildren); assignedTo: alice
      PI-RTH-IMPL-SW-001.md          grandchild, task, leaf, done (ref: evidence)
      PI-RTH-IMPL-SW-002.md          grandchild, task, leaf, todo (no evidence)
      PI-RTH-TEST-001.md             child, task, leaf, blocked (blockedBy: PI-RTH-IMPL-001, no evidence)
      PI-RTH-DOCS-001.md             child, task, leaf, todo (no evidence)
      PI-RTH-BUGFIX-001.md           child, bug, leaf, done (ref: + remote path:)
      PI-RTH-CLOUDLOG-001.md         child, task, leaf, todo, appliesWhen: FEAT-CLOUD-SYNC, assignedTo: bob
  error-demo/
    model/
      _index.md                      Root package (isolated, deliberately-broken)
      Requirements/
        REQ-ERR-DEMO-001.md          placeholder achieves target
      Planning/
        PI-ERR-NOEV-001.md           leaf, done, NO evidence at all -> E719
        PI-ERR-WAIVED-001.md         leaf, done, evidence all rationale:-waived -> E719
```

## What each `PlanningItem` demonstrates

| Item | `status` | `itemType` | `parent` | Leaf? | What it shows |
|---|---|---|---|---|---|
| `PI-RTH-001` | in_progress | feature | — (top-level) | no (5 children) | `achieves:` required + set on a top-level item (id-form list, `REQ-TRS-PLANITEM-003`) |
| `PI-RTH-DESIGN-001` | done | task | `PI-RTH-001` | yes | leaf-evidence rule satisfied by a real local `path:`; a second, `rationale:`-waived, deliberately-dangling `ref:` entry alongside it |
| `PI-RTH-IMPL-001` | in_progress | task | `PI-RTH-001` | no (2 children) | non-leaf: no evidence required regardless of status; `assignedTo: alice` (`REQ-TRS-PLANITEM-008`), resolves to "Alice Nakamura" in `show` |
| `PI-RTH-IMPL-SW-001` | done | task | `PI-RTH-IMPL-001` | yes | 3-level-deep grandchild leaf; `ref:` evidence to a real `TestCase` |
| `PI-RTH-IMPL-SW-002` | todo | task | `PI-RTH-IMPL-001` | yes | leaf with **no** evidence at all — fine, not `done` |
| `PI-RTH-TEST-001` | blocked | task | `PI-RTH-001` | yes | leaf with no evidence — fine, `blocked` isn't `done`; `blockedBy: PI-RTH-IMPL-001` (`REQ-TRS-PLANITEM-007`) |
| `PI-RTH-DOCS-001` | todo | task | `PI-RTH-001` | yes | leaf with no evidence — fine, `todo` isn't `done` |
| `PI-RTH-BUGFIX-001` | done | **bug** | `PI-RTH-001` | yes | `itemType` independent of task-typed siblings; own `achieves:` (qname form, optional on non-top-level); `ref:` + remote-URI `path:` evidence |
| `PI-RTH-CLOUDLOG-001` | todo | task | `PI-RTH-001` | yes | `appliesWhen: FEAT-CLOUD-SYNC` — product-line gating; `assignedTo: bob` (`REQ-TRS-PLANITEM-008`), resolves to "Bob Patel" |

Every `status` value (`todo`/`in_progress`/`blocked`/`done`) and all three
`itemType` values (`task`/`bug`/`feature`) appear at least once.

## Hierarchy (`REQ-TRS-PLANITEM-002`)

```
PI-RTH-001 (feature, in_progress)
├── PI-RTH-DESIGN-001 (task, done)                     leaf
├── PI-RTH-IMPL-001 (task, in_progress)
│   ├── PI-RTH-IMPL-SW-001 (task, done)                leaf
│   └── PI-RTH-IMPL-SW-002 (task, todo)                leaf
├── PI-RTH-TEST-001 (task, blocked)                    leaf
├── PI-RTH-DOCS-001 (task, todo)                       leaf
├── PI-RTH-BUGFIX-001 (bug, done)                      leaf
└── PI-RTH-CLOUDLOG-001 (task, todo)                   leaf
```

The tree is expressed entirely through `parent:`/id relationships, not
directory nesting — every `.md` file above lives flat in `Planning/`. This is
deliberate: unlike SysML `Contains` (directory = namespace), a
`PlanningItem`'s breakdown is a separate, orthogonal structure from the
model's package layout, exactly as `ADR-SYS-PLANITEM-001` describes.

## `achieves:` (`REQ-TRS-PLANITEM-003`)

- `PI-RTH-001` (top-level) → `achieves: [REQ-RTH-001, REQ-RTH-002]` — **id
  form**, a list of two.
- `PI-RTH-BUGFIX-001` (non-top-level, sets it anyway) →
  `achieves: Requirements::REQ-RTH-003` — **qualified-name form**, a single
  scalar.

Both `REQ-RTH-002` and `REQ-RTH-003` are leaf `Requirement`s named only via
`achieves:`, never `satisfies:` — so they still raise `W300` even though
real `PlanningItem` work is targeting them. This is the intended, designed-in
separation between `achieves:` and `satisfies:` (`ADR-SYS-PLANITEM-001`
Decision 2), demonstrated live rather than just asserted: `achieves:` never
suppresses `W300`, and never risks tripping `E312` either (no `PlanningItem`
here ever appears in a `satisfies:` list).

## `blockedBy:` (`REQ-TRS-PLANITEM-007`)

`PI-RTH-TEST-001` (`status: blocked`) sets `blockedBy: PI-RTH-IMPL-001` — the hardware-in-the-loop
verification can't run until the controller logic it exercises is actually done, so the blocker is
the real, structural dependency (`PI-RTH-IMPL-001`, itself `status: in_progress`), not the
hardware-rig-availability detail mentioned in the body text (which names no model element and so
isn't, and shouldn't be, expressed as a `blockedBy:` — resolution requires a real element, and
inventing one just to model an external scheduling fact would be over-fitting). `status` and
`blockedBy:` agree here (`blocked` / non-empty), so this raises no `W308`; if `PI-RTH-IMPL-001`
finishes and `PI-RTH-TEST-001`'s own `status:` is never updated to reflect it, that drift becomes
exactly the `W308` this rule exists to catch — the field is a plain, author-maintained
cross-reference, not computed from anything.

## `assignedTo:` (`REQ-TRS-PLANITEM-008`)

`model/.syscribe.toml` declares a `[users]` roster:

```toml
[users]
alice = "Alice Nakamura"
bob = "Bob Patel"
```

`PI-RTH-IMPL-001` sets `assignedTo: alice`; `PI-RTH-CLOUDLOG-001` sets `assignedTo: bob`. Both are
well-formed Unix-style usernames (`^[a-z_][a-z0-9_-]{0,31}$`, checked unconditionally — `E723`) and
both are declared keys in the roster above, so neither raises `E722` either.
`syscribe show Planning::PI-RTH-IMPL-001` resolves and prints the declared display name alongside
the raw username (`alice (Alice Nakamura)`) — the one display-side effect of the roster being
configured; everything else about `assignedTo:` is schema + validation only, same posture as the
rest of `PlanningItem`. Deleting `.syscribe.toml` from this example (try it) makes roster
membership dormant — both assignments would still validate cleanly on username format alone, with
no `E722` even though nothing is declared, and `show` would print the bare username with no
resolved display name.

## `evidence:` (`REQ-TRS-PLANITEM-005`)

| Item | Entry | Kind | Resolves? |
|---|---|---|---|
| `PI-RTH-IMPL-SW-001` | `ref: TC-RTH-BATT-001` | element ref | yes — real `TestCase` |
| `PI-RTH-BUGFIX-001` | `ref: TC-RTH-NOISE-001` | element ref | yes — real `TestCase` |
| `PI-RTH-BUGFIX-001` | `path: https://github.com/...` | remote URI | yes — accepted as external, no local check |
| `PI-RTH-DESIGN-001` | `path: docs/rth-design.txt` | local file | yes — real file in this example tree |
| `PI-RTH-DESIGN-001` | `ref: PI-RTH-REVIEW-999` + `rationale:` | element ref | **waived** — deliberately dangling, excused by its own `rationale:` |

`PI-RTH-DESIGN-001` shows the waiver and a genuinely-resolving entry
side by side in the same list: the `rationale:`-waived `ref:` never gets
flagged (`E716`), and the `path:` entry alone is what satisfies
`REQ-TRS-PLANITEM-006`'s leaf-evidence rule — proving a waiver excuses one
entry, it doesn't manufacture proof for the list as a whole.

## `appliesWhen:` (`REQ-TRS-PLANITEM-004`)

`Features::CloudSync` (`FEAT-CLOUD-SYNC`) is a standalone optional
`FeatureDef` (no parent group — mirrors this repository's own
`Features::DualFlightController`). `PI-RTH-CLOUDLOG-001` sets
`appliesWhen: FEAT-CLOUD-SYNC`. `CONF-BASE-001` selects the feature `false`;
`CONF-PREMIUM-001` selects it `true`:

```
$ ./target/debug/syscribe -m examples/planning-item/model why-active \
    Planning::PI-RTH-CLOUDLOG-001 --config CONF-BASE-001
Verdict: inactive

$ ./target/debug/syscribe -m examples/planning-item/model why-active \
    Planning::PI-RTH-CLOUDLOG-001 --config CONF-PREMIUM-001
Verdict: active
```

`feature-check --deep`:

```
$ ./target/debug/syscribe -m examples/planning-item/model feature-check --deep
# Feature Model Check

| Code | File | Message |
|---|---|---|
| W022 | examples/planning-item/model/Requirements/REQ-RTH-002.md | requirement 'REQ-RTH-002' is active in some configuration but covered in none |

0 error(s), 1 warning(s)

## Deep analysis
- void model: false
- dead features: none
- core features: none
- false-optional: none
- invalid configurations: none
```

No new gating logic was needed to make this work — `appliesWhen:` is already
fully type-agnostic (confirmed by task #17's tests before this example was
ever built).

## Leaf-evidence rule (`REQ-TRS-PLANITEM-006`)

The success case lives in the main example: `PI-RTH-DESIGN-001`,
`PI-RTH-IMPL-SW-001`, and `PI-RTH-BUGFIX-001` are all leaf, `status: done`,
with at least one genuinely-resolving `evidence:` entry each — zero `E719`
findings.

### Deliberate error demonstration

`error-demo/model/` isolates the rule *firing*, on purpose, so the main
example never has to carry an intentional defect to prove the rule works:

```
$ ./target/debug/syscribe -m examples/planning-item/error-demo/model
...
### Errors

| Code | File | Message |
|---|---|---|
| E719 | examples/planning-item/error-demo/model/Planning/PI-ERR-NOEV-001.md | leaf PlanningItem is `status: done` but has no non-waived, resolving `evidence:` entry |
| E719 | examples/planning-item/error-demo/model/Planning/PI-ERR-WAIVED-001.md | leaf PlanningItem is `status: done` but has no non-waived, resolving `evidence:` entry |

### Warnings

| Code | File | Message |
|---|---|---|
| W001 | examples/planning-item/error-demo/model/Requirements/REQ-ERR-DEMO-001.md | normative text contains no 'shall' |
| W005 | examples/planning-item/error-demo/model/Requirements/REQ-ERR-DEMO-001.md | Requirement 'REQ-ERR-DEMO-001' has no derivedFrom and no derivedChildren — possible orphan |
```

Two distinct failure shapes, both producing the same `E719`:

- **`PI-ERR-NOEV-001`** — `status: done`, no `evidence:` field at all.
- **`PI-ERR-WAIVED-001`** — `status: done`, a non-empty `evidence:` list, but
  *every* entry carries its own `rationale:`. Nothing in the list counts
  toward the "at least one resolving entry" requirement, even though the
  list itself isn't empty — a waiver excuses a check, it doesn't create
  proof.

The `W001`/`W005` warnings are inherent to the placeholder
`REQ-ERR-DEMO-001` (a minimal one-line requirement that exists only so the
demo items have somewhere valid to `achieves:`, not a real requirement worth
polishing) — expected, not a defect in this fixture.

## Expected / documented warnings (main example)

- **`W002`, `W015` × 2 (`REQ-RTH-002` has no covering `TestCase`)** — the
  event-logging requirement genuinely has no test written yet in this
  example (only the battery-trigger and anti-false-trigger paths have
  `TestCase`s); realistic for an `in_progress` feature, not a defect.
- **`W300` × 2 (`REQ-RTH-002`/`REQ-RTH-003` have no satisfying architecture
  element)** — intentional, see "`achieves:` (`REQ-TRS-PLANITEM-003`)" above:
  this is a planning-only demo with no architecture layer, and `achieves:`
  deliberately never counts as `satisfies:`.

## Surprises a real example surfaced that unit tests didn't

- **`syscribe show`'s per-field table has no row for `parent:`, `achieves:`,
  or `itemType:`.** Building `PI-RTH-BUGFIX-001` (which sets both) and
  running `syscribe show Planning::PI-RTH-BUGFIX-001` showed `type`, `file`,
  `id`, `status`, and `evidence` (the last only because it happens to share
  a rendering path with `Argument.evidence`) — but not `parent`, `achieves`,
  or `itemType`, even though every one of those fields validates correctly
  and is fully documented. `crates/syscribe/src/query.rs`'s `cmd_show` has an
  explicit `if let Some(ref x) = fm.<field>` arm per displayable field, and
  none were ever added for `PlanningItem`'s three new fields — an oversight
  from schema/validation work (tasks #14–#19) never touching the CLI display
  layer. No Rust code changed to build this example (out of this task's
  scope); flagging here for a future display-layer follow-up.
- **The `connectivity` command's `--kinds` allowlist doesn't know about
  `planningParent`, or in fact most of the safety/security `EdgeKind`
  variants added over time.** `crates/syscribe/src/connectivity.rs` keeps its
  own hand-maintained `KIND_NAMES` list, separate from
  `syscribe_model::graph::EdgeKind`'s actual variants — running
  `connectivity Planning::PI-RTH-001 --kinds planningParent` fails with
  `unknown edge kind 'planningParent'` even though the edge is real and
  present in the graph (confirmed via `why-active`/`validate` picking up
  `parent:`/`achieves:`/`evidence:` correctly). This is a **general,
  pre-existing gap** — `topEvent`, `hazardousEventRef`,
  `derivedFromSafetyGoal`, and most of the ISO 26262/21434 edge kinds are
  missing from the same list — not something introduced by or unique to
  `PlanningItem`. Noting it here since a real example, not a unit test, is
  what surfaced it; not fixed as part of this task (model content only).
- **A bare relative `path:` resolves against the *model root*, not the
  example's own top-level directory.** `PI-RTH-DESIGN-001`'s
  `path: docs/rth-design.txt` only resolves because `docs/` was placed
  *inside* `model/`, not as a sibling of it — the natural instinct (mirroring
  this README's own placement next to `model/`, not inside it) would have
  put the design note at `examples/planning-item/docs/rth-design.txt` and
  silently failed to resolve as a bare path (it would need a `repo:`-prefixed
  path instead, resolving against the detected git root, which only works
  because this example happens to be nested inside a git checkout — not
  guaranteed for every standalone model root). Worth knowing when authoring
  `evidence:`/`implementedBy:` paths: "bare relative" means model-root
  relative, full stop, matching `config.classify_source`'s documented
  precedence (`model:`/`repo:`/absolute/bare) but easy to get wrong on first
  attempt for anything living outside the model tree.
