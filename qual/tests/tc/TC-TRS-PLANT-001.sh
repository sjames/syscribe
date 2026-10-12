tc_TRS_PLANT_001() {
    local F="$1"
    local G="$F/TC-TRS-PLANT-001/good"
    local B="$F/TC-TRS-PLANT-001/bad"

    SCENARIO_NAME="a lag in a closed loop runs and is deterministic"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_output_contains "0 errors"
    assert_no_code "I011"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" simulate TC-PL-001 2>&1; echo "exit=$?")
    assert_output_contains "3/3 scenario(s) pass"
    assert_output_contains "exit=0"
    local A Bj
    A=$("$SYSCRIBE" -m "$G" simulate TC-PL-001 --json 2>&1 || true)
    Bj=$("$SYSCRIBE" -m "$G" simulate TC-PL-001 --json 2>&1 || true)
    if [ "$A" = "$Bj" ]; then SCENARIO_OUTPUT="identical"; else SCENARIO_OUTPUT="differ"; fi
    assert_output_contains "identical"

    SCENARIO_NAME="malformed plants are reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_count "E935" 1
    assert_output_contains "derivative of 'x'"
    assert_count "E938" 2
    assert_output_contains "algebraic loop"
    assert_count "E939" 1
    assert_output_contains "'Lag2' is not a library plant"
}
