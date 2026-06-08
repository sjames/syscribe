tc_TRS_SPEC_001() {
    # The `spec` subcommand prints embedded reference docs; it needs no model.

    # spec types — safety/security element types (the gap was FMEAEntry).
    SCENARIO_NAME="spec types lists the safety/security types incl. FMEAEntry"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local TYPES; TYPES=$("$SYSCRIBE" spec types 2>/dev/null)
    for t in HazardousEvent SafetyGoal FaultTree FaultTreeGate FaultTreeEvent FMEASheet FMEAEntry \
             TARASheet DamageScenario ThreatScenario CybersecurityGoal SecurityControl VulnerabilityReport; do
        printf '%s' "$TYPES" | grep -qF "$t" && pass "type $t in spec types" || fail "type $t MISSING from spec types"
    done

    # spec fields — safety analysis fields.
    SCENARIO_NAME="spec fields documents the safety analysis fields"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local FIELDS; FIELDS=$("$SYSCRIBE" spec fields 2>/dev/null)
    for f in severity exposure controllability operationalSituation consequence freqExposure avoidance \
             demandRate safeState ftti hazardousEvents topEvent missionTime gateType inputs eventKind \
             failureRate probability entries failureMode effect cause fmeaSeverity occurrence detection \
             rpn recommendedAction; do
        printf '%s' "$FIELDS" | grep -qF "$f" && pass "field $f in spec fields" || fail "field $f MISSING from spec fields"
    done

    # spec fields — security analysis fields.
    SCENARIO_NAME="spec fields documents the security analysis fields"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    for f in damageTable threatTable goalTable controlTable damageSeverity impactCategories attackFeasibility \
             attackVector damageScenarios calLevel securityProperty threatScenarios controlType \
             implementsGoals cvssScore cveId affectedElements mitigatedBy; do
        printf '%s' "$FIELDS" | grep -qF "$f" && pass "field $f in spec fields" || fail "field $f MISSING from spec fields"
    done

    # spec safety — the three fields that were missing from the narrative.
    SCENARIO_NAME="spec safety documents cveId, safeState and ftti"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local SAFETY; SAFETY=$("$SYSCRIBE" spec safety 2>/dev/null)
    for f in cveId safeState ftti; do
        printf '%s' "$SAFETY" | grep -qF "$f" && pass "field $f in spec safety" || fail "field $f MISSING from spec safety"
    done
}
