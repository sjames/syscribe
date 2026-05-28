---
type: Package
name: Requirements
---

This package contains the complete requirements tree for the Engine ECU, structured in three
sub-packages aligned with the three engineering domains: **Performance**, **Safety**, and **Security**.

## Tree structure

```
Requirements
├── REQ-ENG-SYS-000          (stakeholder-level system goal)
├── Performance/             (throttle response and fuel efficiency)
├── Safety/                  (ASIL A–D, derived from HARA safety goals)
└── Security/                (cybersecurity, derived from TARA goals)
```

The breakdown from `REQ-ENG-SYS-000` to each sub-tree is documented in `Decisions/ADR-ENG-SYS-001`.
Every further breakdown between tree levels also carries a `breakdownAdr:` reference.

## Traceability

All requirements trace upward through `derivedFrom:` links to `REQ-ENG-SYS-000`, and
downward to architecture elements via `satisfies:` on `Part`/`PartDef` elements in the
`System` and `Vehicle` packages. Reverse indices (`verifiedBy`, `derivedChildren`) are
computed at load time and not stored in source files.

Each leaf requirement at `status: approved` must have exactly one satisfying architecture
element (warning W300 if none, W301 if more than one). All safety leaf requirements carry
`asilLevel:` and `verificationMethod:`.
