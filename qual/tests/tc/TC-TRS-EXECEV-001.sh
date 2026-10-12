tc_TRS_EXECEV_001() {
    local F="$1"
    local G="$F/TC-TRS-EXECEV-001/good"
    local B="$F/TC-TRS-EXECEV-001/bad"

    SCENARIO_NAME="the evidence of each requirement and element is reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_no_code "W994"
    assert_no_code "W995"
    assert_no_code "W047"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" requirement-coverage 2>&1 || true)
    assert_output_contains "REQ-EC-001  executable"
    assert_output_contains "REQ-EC-002  executable"
    assert_output_contains "REQ-EC-003  descriptive-only"
    assert_output_contains "REQ-EC-004  unverified"
    assert_output_contains "REQ-EC-005  not-applicable"
    assert_output_contains "5 requirement(s): 2 executable, 0 executable-weak, 1 descriptive-only, 1 unverified, 1 not applicable. Executable: 50.0% of 4 applicable."
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" requirement-coverage REQ-EC-001 2>&1 || true)
    assert_output_contains "scenario  TC-EC-001 / go runs  [pass]"
    assert_output_contains "coverage  Exec::CtlM: states 2/3, transitions 1/3"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" exec-coverage 2>&1 || true)
    assert_output_contains "Arch::Ctl [software]  executable; 2 scenario(s); requirements: 2 executable, 0 descriptive-only, 0 unverified"
    assert_output_contains "Arch::Other [software]  not modelled; 0 scenario(s)"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" requirement-coverage --require REQ-EC-003 2>&1; echo "exit=$?")
    assert_output_contains "REQ-EC-003 is descriptive-only, not executable"
    assert_output_contains "exit=2"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" exec-coverage --min-modelled 60 2>&1; echo "exit=$?")
    assert_output_contains "exit=2"

    SCENARIO_NAME="tag defects are reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_count "W994" 1
    assert_output_contains "scenario 'go runs' claims REQ-EC-004"
    assert_count "W995" 1
    assert_output_contains "TC-EC-001 verifies REQ-EC-001, but every scenario is tagged and none claims it"
}
