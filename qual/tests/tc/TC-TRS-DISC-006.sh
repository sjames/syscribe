tc_TRS_DISC_006() {
    local F="$1"; local B="$F/TC-TRS-DISC-006/orphan"

    SCENARIO_NAME="feature-check emits exactly one W024 naming the orphan"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$B" feature-check 2>/dev/null || true)
    SCENARIO_OUTPUT="$out"
    assert_has_code "W024"
    assert_count "W024" 1
    # the W024 row names the orphan (Telematics) and NOT the non-orphans (Wifi, Audio)
    printf '%s' "$out" | grep -F "| W024 |" | grep -qF "Telematics" \
        && pass "W024 names the orphan Telematics" || fail "W024 does not name Telematics"
    for nm in Wifi Audio; do
        printf '%s' "$out" | grep -F "| W024 |" | grep -qF "$nm" \
            && fail "W024 wrongly names $nm" || pass "$nm not flagged by W024"
    done

    SCENARIO_NAME="base validate never emits W024"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    run_scenario "base validate has no W024" "$B"
    assert_no_code "W024"

    SCENARIO_NAME="W024 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" feature-check --deny W024 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 2 ] && pass "feature-check --deny W024 exits 2" || fail "exit ${ec} (expected 2)"
}
