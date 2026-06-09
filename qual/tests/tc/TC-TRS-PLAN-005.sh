tc_TRS_PLAN_005() {
    local F="$1"; local M="$F/TC-TRS-PLAN-005/model"

    SCENARIO_NAME="testplan lists the plan with scope and verdict"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$M" testplan 2>/dev/null || true)
    printf '%s' "$out" | grep -q "TP-NAV-001" && pass "list names the plan" || fail "plan not listed"
    printf '%s' "$out" | grep -qi "integration" && pass "list shows scope" || fail "scope missing"
    printf '%s' "$out" | grep -qi "pass" && pass "list shows the pass verdict" || fail "verdict missing"

    SCENARIO_NAME="testplan TP-X --json carries the contract shape"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$M" testplan TP-NAV-001 --json 2>/dev/null || true)
    printf '%s' "$j" | jq -e 'has("schemaVersion") and has("inScopeRequirements") and has("effectiveTestCases") and has("coverage") and has("verdict")' >/dev/null \
        && pass "detail json has the required keys" || fail "detail json missing keys"
    printf '%s' "$j" | jq -e '.verdict == "pass"' >/dev/null && pass "verdict is pass (results ingested)" || fail "verdict not pass"
    printf '%s' "$j" | jq -e '.inScopeRequirements | (index("REQ-NAV-001") != null) and (index("REQ-NAV-002") != null)' >/dev/null \
        && pass "in-scope = goal-closure (parent + leaf)" || fail "goal-closure in-scope wrong"
    printf '%s' "$j" | jq -e '[.effectiveTestCases[].id] | index("TC-NAV-001") != null' >/dev/null \
        && pass "effective members include the explicit member" || fail "member missing"

    SCENARIO_NAME="testplan on an unknown plan exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" testplan TP-NOPE-001 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "unknown plan exits non-zero" || fail "unknown plan did not error"
}
