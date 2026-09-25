tc_TRS_OUT_001() {
    local F="$1"
    run_scenario "report is written to stdout in Markdown format" "$F/valid-model"
    assert_output_contains "# "
    assert_stdout_nonempty
    assert_exit_zero

    run_scenario "report title names the model root package (GH #174)" "$F/valid-model"
    [ "$(head -n1 <<<"$SCENARIO_OUTPUT")" = "# ValidModel Validation Report" ] \
        && pass "title is '# ValidModel Validation Report'" \
        || fail "title is '$(head -n1 <<<"$SCENARIO_OUTPUT")'"
    grep -qF "UAV" <<<"$SCENARIO_OUTPUT" && fail "report mentions UAV" || pass "report does not mention UAV"

    run_scenario "report title falls back to a neutral heading (GH #174)" "$F/TC-TRS-OUT-001/unnamed"
    [ "$(head -n1 <<<"$SCENARIO_OUTPUT")" = "# Model Validation Report" ] \
        && pass "title is '# Model Validation Report'" \
        || fail "title is '$(head -n1 <<<"$SCENARIO_OUTPUT")'"
}
