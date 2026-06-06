tc_TRS_VAL_011() {
    local F="$1"
    local SRC="$F/TC-TRS-VAL-011/scaffold"

    # Scenario 1: E106 message is actionable (carries the exact Scenario: line).
    run_scenario "E106 message is actionable" "$SRC"
    assert_has_code "E106"
    assert_output_contains "Scenario: Case B passes"

    # Scenario 2: scaffold-gherkin --fix aligns the block on a working copy and clears E106.
    SCENARIO_NAME="scaffold-gherkin --fix clears E106"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local work; work=$(mktemp -d)
    cp -r "$SRC"/. "$work"/
    "$SYSCRIBE" -m "$work" scaffold-gherkin TC-SCAF-FIX-001 --fix >/dev/null 2>&1
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$work" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_no_code "E106"
    rm -rf "$work"
}
