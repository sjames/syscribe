tc_TRS_SEC_003() {
    local F="$1"; local B="$F/TC-TRS-SEC-003"

    # The worked example: AT-DEMO-001 rolls up to medium (OR of [AND(min(high,low)=low),
    # medium] = max(low,medium) = medium) while TS-DEMO-001 declares high — so W035
    # fires exactly once, naming computed medium vs declared high, and the model
    # validates with no errors and no dangling/orphan refs for the attack-tree types.
    run_scenario "worked example rolls up to medium; W035 vs declared high; no errors" "$B/main"
    assert_exit_zero
    assert_has_code "W035"
    assert_count "W035" 1
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W035 |" | grep -qF "AT-DEMO-001" \
        && pass "W035 names the AttackTree AT-DEMO-001" || fail "W035 not on AT-DEMO-001"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W035 |" | grep -qiF "medium" \
        && pass "W035 names computed medium" || fail "W035 missing computed medium"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W035 |" | grep -qiF "high" \
        && pass "W035 names declared high" || fail "W035 missing declared high"
    # No structural errors for the attack-tree types.
    assert_no_code "E915"; assert_no_code "E916"; assert_no_code "E917"
    assert_no_code "E918"; assert_no_code "E919"; assert_no_code "E920"
    assert_no_code "E921"; assert_no_code "W036"

    # Aligning the declared feasibility (TS = medium) clears W035, still no errors.
    run_scenario "matched declared feasibility clears W035" "$B/matched"
    assert_exit_zero
    assert_no_code "W035"

    # threatRef to a non-ThreatScenario produces E917.
    run_scenario "threatRef to a non-ThreatScenario produces E917" "$B/badref"
    assert_has_code "E917"

    # W035 is gateable with --deny.
    SCENARIO_NAME="W035 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/main" validate --deny W035 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W035 exits non-zero" || fail "--deny W035 did not gate"

    # The `types` command lists the three new element types.
    SCENARIO_NAME="types command lists the new attack-tree types"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local t; t=$("$SYSCRIBE" -m "$B/main" types 2>/dev/null || true)
    printf '%s' "$t" | grep -qF "AttackTree" \
        && pass "types lists AttackTree" || fail "types missing AttackTree"
    printf '%s' "$t" | grep -qF "AttackTreeGate" \
        && pass "types lists AttackTreeGate" || fail "types missing AttackTreeGate"
    printf '%s' "$t" | grep -qF "AttackStep" \
        && pass "types lists AttackStep" || fail "types missing AttackStep"
}
