tc_TRS_TRACE_010() {
    local F="$1"; local B="$F/TC-TRS-TRACE-010"

    # flagged: two SIL-4 requirements (draft+unsat, approved+unsat) fire W306;
    # the SIL-2 one is below threshold and must not.
    run_scenario "high-integrity draft/unsatisfied requirements produce W306" "$B/flagged"
    assert_has_code "W306"
    assert_count "W306" 2

    # The message names the triggering sub-condition (draft / unsatisfied).
    SCENARIO_NAME="W306 message names the sub-conditions"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local OUT; OUT=$("$SYSCRIBE" -m "$B/flagged" validate 2>/dev/null)
    printf '%s' "$OUT" | grep "W306" | grep -qiE "draft" \
        && printf '%s' "$OUT" | grep "W306" | grep -qiE "satisf" \
        && pass "W306 messages name draft and unsatisfied" || fail "W306 message does not name sub-conditions"

    # ok: a non-draft, satisfied SIL-4 requirement produces no W306.
    run_scenario "a fully-integrated high-integrity requirement produces no W306" "$B/ok"
    assert_no_code "W306"

    # gateable
    SCENARIO_NAME="W306 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/flagged" validate --deny W306 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W306 exits non-zero" || fail "--deny W306 did not gate"
}
