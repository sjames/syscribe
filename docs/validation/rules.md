# Rule Reference

`VALIDATION · RULES`

## Parse-time errors (E001–E015)

| Code | Element | Condition |
|---|---|---|
| E004 | TestCase | `id`, `title`, `status`, or `testLevel` absent |
| E004 | Requirement | `title` or `status` absent on native Requirement |
| E006 | Requirement | `id` present but does not match `REQ-*` pattern |
| E006 | TestCase | `id` present but does not match `TC-*` pattern |
| E007 | Requirement | `status` is not one of `draft · review · approved · implemented · verified` |
| E007 | TestCase | `status` is not one of `draft · review · approved · active · retired` |
| E008 | TestCase | `testLevel` is not one of `L1 · L2 · L3 · L4 · L5` |
| E009 | Any | `silLevel` is not in range 1–4 |
| E010 | Any | `asilLevel` is not one of `A · B · C · D` |
| E011 | TestCase | Body has no ` ```gherkin ` fenced block |
| E012 | Requirement | Normative text (before first `##`) is empty |
| E013 | TestCase | `verifies:` is absent or empty |
| E014 | TestCase | `Scenario Outline:` block has no `Examples:` table |
| E015 | TestCase | First ` ```gherkin ` block has no `Feature:` line |

## Parse-time warnings (W001–W006)

| Code | Condition |
|---|---|
| W001 | Requirement normative text contains no `shall` |
| W004 | `sourceFile:` path does not exist on disk |
| W006 | `silLevel` present without `asilLevel`, or vice versa |

## Cross-reference errors (E101–E106)

| Code | Condition |
|---|---|
| E101 | Duplicate `id` across two elements |
| E102 | `verifies:` entry does not resolve |
| E103 | `derivedFrom:` entry does not resolve |
| E104 | `verifies:` target is not a native Requirement |
| E105 | `derivedFrom:` target is not a native Requirement |
| E106 | `testFunctions[].scenario` name not found in Gherkin blocks |

## Coverage warnings (W002–W005)

| Code | Condition |
|---|---|
| W002 | Requirement at `approved` or `implemented` has no active TestCase |
| W003 | Requirement at `verified` has no active TestCase covering it |
| W005 | Requirement has no `derivedFrom` and no `derivedChildren` — possible orphan |

## PLE errors (E200–E209)

| Code | Condition |
|---|---|
| E200 | Configuration `id` does not match `CONF-*` pattern |
| E201 | Configuration missing `id`, `title`, `status`, or `featureModel` |
| E209 | `appliesWhen:` entry does not resolve to a FeatureDef |

## ADR errors (E300–E304)

| Code | Condition |
|---|---|
| E300 | ADR `id` does not match `ADR-*` pattern |
| E301 | ADR missing `id`, `title`, or `status` |
| E302 | `reqDomain` value is not `system · hardware · software` |
| E303 | `domain` value is not `system · hardware · software` |
| E304 | ADR `status` is not `proposed · accepted · deprecated · superseded` |

## Traceability warnings (W300–W304)

| Code | Condition |
|---|---|
| W300 | Leaf Requirement at `approved` or `implemented` has no satisfying element |
| W301 | Leaf Requirement is satisfied by more than one element |
| W302 | Leaf Requirement at `implemented` or `verified` still has `reqDomain: system` |
| W303 | `breakdownAdr:` references a `proposed` ADR but Requirement is `approved` or higher |
| W304 | `isDeploymentPackage: true` combined with `domain: hardware` |

## §12 Traceability errors (E310–E315)

| Code | Condition |
|---|---|
| E310 | Requirement has `derivedFrom:` but no `breakdownAdr:` |
| E311 | `breakdownAdr:` does not resolve, or does not resolve to an ADR |
| E312 | Parent requirement (has `derivedChildren`) appears in a `satisfies:` list |
| E313 | `satisfies` domain mismatch: element domain ≠ requirement `reqDomain` |
| E314 | `isDeploymentPackage: true` element has no Allocation to a hardware element |
| E315 | Cross-domain `supertype:` or `typedBy:` reference — use Allocation instead |

## Diagram errors (E400–E402)

| Code | Condition |
|---|---|
| E400 | `diagramKind: Mermaid` but body has no ` ```mermaid ` block |
| E401 | `diagramKind: PlantUML` but body has no ` ```plantuml ` block |
| E402 | `svgFile:` path does not exist on disk |

## Diagram warnings (W400–W403)

| Code | Condition |
|---|---|
| W400 | Diagram has no `diagramKind` — rendering mode ambiguous |
| W401 | `subject:` does not resolve to a known element |
| W402 | Shape `ref:` does not resolve (and is not a sub-feature of a known element) |
| W403 | Edge `source` or `target` is not a defined shape id in this diagram |

## Allocation errors (E500–E503)

| Code | Condition |
|---|---|
| E500 | Feature with `type: Allocation` has `allocatedFrom:` that does not resolve |
| E501 | Feature with `type: Allocation` has `allocatedTo:` that does not resolve |
| E502 | Top-level `allocatedFrom:` on Allocation element does not resolve |
| E503 | Top-level `allocatedTo:` on Allocation element does not resolve |

## Structural warnings (W500–W502)

| Code | Condition |
|---|---|
| W500 | `viewpoint:` on View does not resolve to a ViewpointDef |
| W501 | `exhibitsStates:` entry does not resolve to any known element |
| W502 | `expose:` entry on View does not resolve to any known element |

## Documentation warnings (W600–W601)

| Code | Condition |
|---|---|
| W600 | PartDef or Part has an empty documentation body |
| W601 | ActionDef or Action has an empty documentation body |

## Operations warning (W404)

| Code | Condition |
|---|---|
| W404 | Operation `typedBy` (parameter) or `returnType` does not resolve to a known element |
