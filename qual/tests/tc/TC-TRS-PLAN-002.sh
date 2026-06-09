tc_TRS_PLAN_002() {
    local F="$1"; local B="$F/TC-TRS-PLAN-002"

    SCENARIO_NAME="a multi-config plan with resolvable configs is clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate 2>/dev/null || true)
    assert_no_code "E606"; assert_no_code "W611"

    SCENARIO_NAME="an unresolvable configurations entry raises E606"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/badconfig" validate 2>/dev/null || true)
    assert_has_code "E606"

    SCENARIO_NAME="a member active in none of the plan's configs raises W611"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/escaping" validate 2>/dev/null || true)
    assert_has_code "W611"

    SCENARIO_NAME="two plans with identical (configurations, scope) raise W616"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/dupplan" validate 2>/dev/null || true)
    assert_has_code "W616"
}
