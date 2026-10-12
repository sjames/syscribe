tc_TRS_VCOMP_001() {
    local F="$1"
    local G="$F/TC-TRS-VCOMP-001/good"
    local B="$F/TC-TRS-VCOMP-001/bad"

    SCENARIO_NAME="a verification composite is clean and its reports separate the environment"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_no_code "E992"
    assert_no_code "W991"
    assert_no_code "W992"
    assert_no_code "W993"
    assert_no_code "E991"
    assert_no_code "W047"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" simulate Exec::Bench --coverage 2>&1 || true)
    assert_output_contains "coverage of Exec::Bench.stim (Exec::StimM) [environment]"
    assert_output_contains "(product parts only; environment parts not counted: stim)"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" inject Exec::Bench --report-only 2>&1 || true)
    assert_output_contains "environment parts (not the product): stim"
    assert_output_contains "FM-VC-001 [environment]"

    SCENARIO_NAME="each defect is reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_count "E992" 1
    assert_output_contains "feature 'spare' has \`role: enviroment\`, which is not a role"
    assert_has_code "W993"
    assert_output_contains "part 'stim' of 'Exec::Bench': is \`role: environment\`"
    assert_has_code "W992"
    assert_output_contains "REQ-VC-003"
    assert_has_code "W991"
    assert_output_contains "connection mon.out → rel.cmd of 'Exec::Bench' has no counterpart in the architecture"
}
