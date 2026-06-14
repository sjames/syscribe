# Appendix C — Quick Reference Card

`GUIDE · APPENDIX C — QUICK REFERENCE`


### Element types by role

```
Requirements          Requirement (REQ-*)
Architecture          PartDef · PortDef · ConnectionDef · Diagram · ADR (ADR-*)
Allocation            Allocation · allocatedTo: field
Tests                 TestCase (TC-*) · TestPlan (TP-*)
Config management     Configuration · FeatureDef · appliesWhen:

Safety (HARA/FTA)     HazardousEvent (HE-*) · SafetyGoal (SG-*)
                      FaultTree (FT-*) · FaultTreeGate (FTG-*) · FaultTreeEvent (FTE-*)
                      FMEASheet (FMEA-*) · Argument (ARG-*) · AssumptionOfUse (AOU-*)
                      ConfirmationMeasure (CM-*)

Security (TARA)       Asset (ASSET-*) · DamageScenario (DS-*)
                      ThreatScenario (TS-*) · CybersecurityGoal (CSG-*)
                      SecurityControl (SC-*) · VulnerabilityReport (VR-*)
                      AttackTree (AT-*) · AttackTreeGate (ATG-*) · AttackStep (ATS-*)
```

### Command cheat sheet

```bash
# Model exploration
syscribe -m model ls                     # list root packages
syscribe -m model tree                   # full namespace tree
syscribe -m model find <name>            # fuzzy search
syscribe -m model show <qname|id>        # element detail
syscribe -m model template Requirement   # paste-ready frontmatter skeleton

# Traceability
syscribe -m model trace REQ-X-001        # full chain: goal→req→arch→test
syscribe -m model who-verifies REQ-X-001 # test cases covering a requirement
syscribe -m model why PartDef::MyComp    # requirements the component satisfies
syscribe -m model links REQ-X-001        # all outbound + inbound edges

# Validation
syscribe -m model validate               # full model check
syscribe -m model validate --deny W015,W031  # promote specific warnings
syscribe -m model validate --profile sil4    # named policy

# Coverage
syscribe -m model matrix                 # full coverage matrix
syscribe -m model matrix --gaps-only     # only rows with gaps
syscribe -m model matrix --status approved   # only approved requirements
syscribe -m model verification-depth --sil 4 --min-levels 2

# Safety
syscribe -m model metrics                # SPFM / LFM / PMHF per goal
syscribe -m model safety-case            # GSN trees for all goals
syscribe -m model safety-case SG-X-001  # GSN tree for one goal
syscribe -m model fmea report FMEA-X-001 # RPN-ranked FMEA table
syscribe -m model fault-tree render FT-X-001  # Mermaid FTA
syscribe -m model audit                  # readiness dashboard

# Security
syscribe -m model cyber-risk             # ISO/SAE 21434 risk matrix
syscribe -m model co-analysis            # safety ↔ security overlap
syscribe -m model testplan TP-SEC-001    # security plan membership

# MagicGrid
syscribe -m model magicgrid              # grid population report
syscribe -m model magicgrid --audit      # grid + PASS/FAIL verdict
syscribe -m model trade-study            # MoE-weighted alternative scoring
syscribe -m model matrix --allocations   # function → structure allocation matrix

# Maintenance
syscribe -m model next-id REQ-SCHED      # next available ID
syscribe -m model check-ref Safety::SG-X-001  # verify a reference resolves
syscribe -m model move <src> <dst> --dry-run  # safe rename preview
syscribe -m model move <src> <dst>       # rename + rewrite all references
syscribe -m model extref "DOORS://..."   # find elements by external reference
```

### Integrity level gating — what syscribe checks

| Standard | Field | W033 gates | Informational only |
|---|---|---|---|
| ISO 26262 ASIL D | `asilLevel: D` | PMHF < 1e-8/h AND SPFM ≥ 0.99 AND LFM ≥ 0.90 | — |
| ISO 26262 ASIL C | `asilLevel: C` | PMHF < 1e-7/h AND SPFM ≥ 0.97 AND LFM ≥ 0.80 | — |
| IEC 61508 SIL 4 | `silLevel: 4` | PMHF/PFH < 1e-8/h | SPFM, LFM |
| IEC 61508 SIL 3 | `silLevel: 3` | PMHF/PFH < 1e-7/h | SPFM, LFM |
| ISO/SAE 21434 | `calLevel: CAL4` | CAL ≥ risk-implied minimum (W032) | No numeric PFH |
