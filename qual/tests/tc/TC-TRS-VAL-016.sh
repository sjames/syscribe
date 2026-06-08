tc_TRS_VAL_016() {
    local F="$1"; local B="$F/TC-TRS-VAL-016"

    # list --has-wcet keeps only requirements that declare wcet.
    SCENARIO_NAME="list --has-wcet keeps only wcet-bearing requirements"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local OUT; OUT=$("$SYSCRIBE" -m "$B/flagged" list Requirement --has-wcet 2>/dev/null)
    printf '%s' "$OUT" | grep -qF "REQ-WCET-001" && printf '%s' "$OUT" | grep -qF "REQ-NOSIL-002" \
        && ! printf '%s' "$OUT" | grep -qF "REQ-PLAIN-003" \
        && pass "only wcet requirements listed" || fail "--has-wcet filter wrong"

    # list --json includes the wcet value.
    SCENARIO_NAME="list --has-wcet --json includes wcet"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    OUT=$("$SYSCRIBE" -m "$B/flagged" list Requirement --has-wcet --json 2>/dev/null)
    printf '%s' "$OUT" | grep -qF '"wcet"' && printf '%s' "$OUT" | grep -qF "10ms" \
        && pass "wcet present in list --json" || fail "wcet missing from list --json"

    # W029 fires for a SIL requirement with wcet and only a non-measuring (L3) test.
    run_scenario "SIL requirement with wcet, no measuring test produces W029" "$B/flagged"
    assert_has_code "W029"
    assert_count "W029" 1   # only REQ-WCET-001; REQ-NOSIL-002 (no sil) and REQ-PLAIN-003 (draft, no wcet) excluded

    # An active L5/timing-tagged test clears W029.
    run_scenario "a measuring test clears W029" "$B/measured"
    assert_no_code "W029"

    # W029 is gateable.
    SCENARIO_NAME="W029 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/flagged" validate --deny W029 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -ne 0 ] && pass "validate --deny W029 exits non-zero" || fail "--deny W029 did not gate"
}
