tc_TRS_FM_002() {
    local F="$1"; local B="$F/TC-TRS-FM-002"

    SCENARIO_NAME="structural violations emit E212, E219, E220, W011, W012"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/violations" feature-check 2>/dev/null || true)
    assert_has_code "E212"
    assert_has_code "E219"
    assert_has_code "E220"
    assert_has_code "W011"
    assert_has_code "W012"

    SCENARIO_NAME="clean feature model emits none of them"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/clean" feature-check 2>/dev/null || true)
    assert_no_code "E212"
    assert_no_code "E219"
    assert_no_code "E220"
    assert_no_code "W011"
    assert_no_code "W012"
}
