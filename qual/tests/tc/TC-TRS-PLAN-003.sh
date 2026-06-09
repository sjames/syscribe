tc_TRS_PLAN_003() {
    local F="$1"; local B="$F/TC-TRS-PLAN-003"

    SCENARIO_NAME="an unresolvable testCases entry raises E601"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/badmember" validate 2>/dev/null || true)
    assert_has_code "E601"

    SCENARIO_NAME="a selection.testLevels outside L1-L5 raises E602"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/badlevel" validate 2>/dev/null || true)
    assert_has_code "E602"

    SCENARIO_NAME="a selection.domains outside system/hardware/software raises E605"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/baddomain" validate 2>/dev/null || true)
    assert_has_code "E605"

    SCENARIO_NAME="an empty effective TestCase set raises W612"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/empty" validate 2>/dev/null || true)
    assert_has_code "W612"

    SCENARIO_NAME="an explicitly named draft TestCase raises W613"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/draftmember" validate 2>/dev/null || true)
    assert_has_code "W613"
}
