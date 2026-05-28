# Syscribe Namespace, Cross-References, and Multiplicity

## Directory and namespace conventions (§4)

### Layout

```
model/
  _index.md              # type: Namespace  — root namespace (optional)
  PackageA/
    _index.md            # type: Package, name: PackageA  (optional)
    Foo.md               # qname: PackageA::Foo
    Sub/
      _index.md          # type: Package
      Bar.md             # qname: PackageA::Sub::Bar
  Requirements/
    Req001.md            # qname: Requirements::Req001
```

### Key rules

1. **Qualified name** = `::` joined directory path from model root, using `name:` override if present.
2. `_index.md` represents the directory's package — it has **no own QName segment**.
3. `name:` in frontmatter overrides the filename stem as the QName segment and display name.
4. Two elements in the same directory must not have the same effective `name:` (E-collision).
5. `visibility: private` restricts membership; default is public.
6. Library packages (`type: LibraryPackage`) auto-export all members.

### `_index.md` fields

| Field | Type | Description |
|---|---|---|
| `type` | string | `Package` · `LibraryPackage` · `Namespace` |
| `name` | string | Override directory name |
| `imports` | list | Import declarations |
| `aliases` | list | Alias declarations |
| `filterCondition` | string | Package filter (KerML opaque expression) |

### Standard library packages

**Auto-imported (always available):**

| Package | Contents |
|---|---|
| `ScalarValues` | `Integer`, `Real`, `String`, `Boolean`, `Natural` |
| `Base` | `Anything`, `DataValue` |

**Available via explicit `imports:`:**

| Package | Contents |
|---|---|
| `ISQ` | International System of Quantities |
| `SI` | SI units |
| `Parts` | Base `Part` |
| `Items` | Base `Item` |
| `Ports` | Base `Port` |
| `Actions` | Base `Action` |
| `States` | Base `StateAction` |
| `Calculations` | Base `Calculation` |
| `Constraints` | Base `Constraint` |
| `Requirements` | Base `Requirement` |
| `Allocations` | Base `Allocation` |
| `Connections` | Base `Connection`, `Interface` |
| `Flows` | Base `FlowConnection` |
| `Transfers` | Base `Transfer` |
| `Occurrences` | Base `Occurrence`, `EventOccurrence` |
| `Events` | Event definitions |
| `UseCases` | Base `UseCase` |
| `AnalysisCases` | Base `AnalysisCase` |
| `VerificationCases` | Base `VerificationCase`, `VerdictKind` |
| `Views` | Base `View`, `Viewpoint`, `Rendering` |
| `Metadata` | Base `SemanticMetadata` |
| `ModelingMetadata` | `StatusInfo`, `Issue`, `Rationale`, `Refinement` |

---

## Cross-reference syntax (§5)

### Absolute qualified names

```yaml
typedBy: VehicleSystem::Powertrain::Engine
supertype: Requirements::FunctionalRequirementDef
```

### Relative references

Unqualified name resolves within the current package first, then outward:

```yaml
typedBy: Engine          # looks in same directory first
```

`./` prefix means sibling in the same directory:

```yaml
subsets: [./drivetrainPort]
```

### Wildcard imports

| Syntax | Meaning |
|---|---|
| `PackageName` | Import single named element |
| `PackageName::*` | Import all visible members |
| `PackageName::**` | Recursively import all sub-namespaces |

### Aliases

```yaml
aliases:
  - name: MV
    for: ISQ::MassValue
```

### Feature chains (dot notation)

Used in `connections:`, `flowConnections:`, `successionConnections:`:

```yaml
connections:
  - from: engine.exhaustPort
    to: exhaustSystem.inletPort
```

### Native-ID resolution

`verifies:`, `derivedFrom:`, `derivedFromSafetyGoal:`, etc. accept stable IDs (`REQ-*`,
`TC-*`, `SG-*`, etc.) as well as qualified names. IDs are resolved globally across the
entire model — they do not depend on file location.

---

## Multiplicity syntax (§6)

The `multiplicity:` field is a **quoted string**:

| Value | Meaning |
|---|---|
| `"1"` | Exactly 1 |
| `"0"` | Zero |
| `"*"` | Zero or more (0..*) |
| `"0..1"` | Optional |
| `"1..*"` | One or more |
| `"0..*"` | Zero or more |
| `"2..4"` | Between 2 and 4 |
| `"0..maxSeats"` | Upper bound from a feature reference |

Rules: always quote; `*` may not be the lower bound; unresolved feature-ref bounds are errors.

---

## Inline vs separate file (§7)

### Use inline `features:` when

- Simple scalar attribute with primitive/library type, no sub-features.
- Feature used only within this one definition.
- No significant documentation, no own metadata.

### Use a separate `.md` file when

- Feature has sub-features (nested parts, ports, attributes).
- Feature is a `PortDef`, `ActionDef`, `StateDef`, or other reusable definition.
- Feature has significant documentation or own metadata.
- Feature must be referenced by qualified name from other files.
