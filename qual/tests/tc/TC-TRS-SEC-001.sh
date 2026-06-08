tc_TRS_SEC_001() {
    local F="$1"; local B="$F/TC-TRS-SEC-001"

    # Linked safety-tagged DamageScenario validates with no errors; W030 fires
    # only for the unlinked one, not the linked one.
    run_scenario "linked safety damage scenario: no errors, W030 only on the unlinked one" "$B/clean"
    assert_exit_zero
    assert_has_code "W030"
    # exactly one W030 (DS-SEC-002 only; DS-SEC-001 is linked)
    assert_count "W030" 1
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W030 |" | grep -qF "DS-SEC-002" \
        && pass "W030 names the unlinked DS-SEC-002" || fail "W030 not on DS-SEC-002"
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W030 |" | grep -qF "DS-SEC-001" \
        && fail "W030 wrongly fired on linked DS-SEC-001" || pass "no W030 on linked DS-SEC-001"

    # Bad hazardRef (unresolved + wrong-type) produces E844.
    run_scenario "unresolved or wrong-type hazardRef produces E844" "$B/badref"
    assert_has_code "E844"

    # W030 is gateable.
    SCENARIO_NAME="W030 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/clean" validate --deny W030 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W030 exits non-zero" || fail "--deny W030 did not gate"

    # co-analysis text names the SafetyGoal and its ThreatScenario.
    SCENARIO_NAME="co-analysis text names the safety goal and its threat"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local t; t=$("$SYSCRIBE" -m "$B/clean" co-analysis 2>/dev/null || true)
    printf '%s' "$t" | grep -qF "SG-SEC-001" && pass "names SafetyGoal SG-SEC-001" || fail "missing SG-SEC-001"
    printf '%s' "$t" | grep -qF "TS-SEC-001" && pass "names ThreatScenario TS-SEC-001" || fail "missing TS-SEC-001"
    printf '%s' "$t" | grep -qF "DS-SEC-002" && pass "lists unlinked DS-SEC-002 gap" || fail "missing DS-SEC-002 gap"

    # co-analysis --json is valid JSON with goals and unlinkedSafetyDamage.
    SCENARIO_NAME="co-analysis --json carries goals and unlinkedSafetyDamage"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$B/clean" co-analysis --json 2>/dev/null || true)
    printf '%s' "$j" | jq -e '.goals | length >= 1' >/dev/null 2>&1 \
        && pass "json has goals" || fail "json missing goals"
    printf '%s' "$j" | jq -e '.goals[0].id == "SG-SEC-001"' >/dev/null 2>&1 \
        && pass "goal id is SG-SEC-001" || fail "goal id not SG-SEC-001"
    printf '%s' "$j" | jq -e '.goals[0].threats[0].id == "TS-SEC-001"' >/dev/null 2>&1 \
        && pass "goal threat is TS-SEC-001" || fail "goal threat not TS-SEC-001"
    printf '%s' "$j" | jq -e '.unlinkedSafetyDamage[0].id == "DS-SEC-002"' >/dev/null 2>&1 \
        && pass "unlinkedSafetyDamage lists DS-SEC-002" || fail "unlinkedSafetyDamage missing DS-SEC-002"
}
