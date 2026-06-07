tc_TRS_PARAM_002() {
    local F="$1"; local B="$F/TC-TRS-PARAM-002"

    SCENARIO_NAME="violations: E221 + W025 fire; compound appliesWhen parsed (no W014)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/violations" feature-check 2>/dev/null || true)
    assert_has_code "E221"
    assert_has_code "W025"
    assert_has_code "E213"          # PC-GHOST unresolved path still detected
    assert_no_code  "W014"          # compound "A and B" must NOT be a bogus unknown feature
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| E221 |" | grep -qF "CONF-PC-VIOL-001" \
        && pass "E221 names the violating configuration" || fail "E221 does not name CONF-PC-VIOL-001"

    SCENARIO_NAME="clean: satisfying + non-applicable configs are silent"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/clean" feature-check 2>/dev/null || true)
    assert_no_code "E221"
    assert_no_code "W025"
    assert_no_code "W014"

    SCENARIO_NAME="W025 is gateable via --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/violations" feature-check --deny W025 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "--deny W025 exits non-zero" || fail "--deny W025 did not gate"
}
