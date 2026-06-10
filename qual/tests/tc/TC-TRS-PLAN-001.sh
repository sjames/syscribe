tc_TRS_PLAN_001() {
    local F="$1"; local B="$F/TC-TRS-PLAN-001"

    SCENARIO_NAME="a well-formed TestPlan parses clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/valid" validate 2>/dev/null || true)
    "$SYSCRIBE" -m "$B/valid" validate >/dev/null 2>&1 && pass "valid plan validate exit 0" \
        || fail "valid plan validate exited non-zero"
    assert_no_code "E600"; assert_no_code "E604"; assert_no_code "W610"

    SCENARIO_NAME="schema violations raise E600 / E604 / W610"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/bad" validate 2>/dev/null || true)
    assert_has_code "E600"
    assert_has_code "E604"
    assert_has_code "W610"

    SCENARIO_NAME="duplicate TestPlan id raises the generic E101"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/dup" validate 2>/dev/null || true)
    assert_has_code "E101"

    SCENARIO_NAME="template TestPlan emits a valid skeleton"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local tpl; tpl=$("$SYSCRIBE" -m "$B/valid" template TestPlan 2>/dev/null || true)
    printf '%s' "$tpl" | grep -q "type: TestPlan" && pass "skeleton has type: TestPlan" || fail "no type: TestPlan"
    printf '%s' "$tpl" | grep -qE "id: TP-" && pass "skeleton has a TP-* id" || fail "no TP-* id"
    printf '%s' "$tpl" | grep -q "scope:" && printf '%s' "$tpl" | grep -q "testCases:" \
        && pass "skeleton has scope and testCases" || fail "skeleton missing scope/testCases"
}
