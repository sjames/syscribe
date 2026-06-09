tc_TRS_PLAN_004() {
    local F="$1"; local B="$F/TC-TRS-PLAN-004"

    SCENARIO_NAME="an unresolvable demonstrates target raises E603"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/baddemo" validate 2>/dev/null || true)
    assert_has_code "E603"

    SCENARIO_NAME="an approved plan demonstrating an unverified requirement raises W614"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/gap" validate 2>/dev/null || true)
    assert_has_code "W614"

    SCENARIO_NAME="a demonstrated-and-covered plan is clean of E603/W614"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate 2>/dev/null || true)
    assert_no_code "E603"; assert_no_code "W614"
}
