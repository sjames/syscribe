tc_TRS_IMPL_001() {
    local F="$1"; local B="$F/TC-TRS-IMPL-001/model"
    run_scenario "implementedBy missing path -> W023 (opt-in, draft-suppressed)" "$B"
    assert_has_code "W023"
    assert_count "W023" 1
    # the one finding is the non-draft missing element; not the others
    printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W023 |" | grep -qF "Ghost.md" \
        && pass "W023 names the missing-path element" || fail "W023 does not name Ghost"
    for nm in Sched.md Draft.md Plain.md; do
        printf '%s' "$SCENARIO_OUTPUT" | grep -F "| W023 |" | grep -qF "$nm" \
            && fail "W023 wrongly flags $nm" || pass "$nm not flagged"
    done
    SCENARIO_NAME="W023 is gateable with --deny"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B" validate --deny W023 >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 2 ] && pass "--deny W023 exits 2" || fail "exit ${ec} (expected 2)"
}
