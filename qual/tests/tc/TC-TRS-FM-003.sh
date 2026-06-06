tc_TRS_FM_003() {
    local F="$1"; local B="$F/TC-TRS-FM-003"

    SCENARIO_NAME="parameter violations emit E207, E202, E213, W014"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/violations" feature-check 2>/dev/null || true)
    assert_has_code "E207"
    assert_has_code "E202"
    assert_has_code "E213"
    assert_has_code "W014"

    SCENARIO_NAME="clean feature model emits none of them"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/clean" feature-check 2>/dev/null || true)
    assert_no_code "E207"
    assert_no_code "E202"
    assert_no_code "E213"
    assert_no_code "W014"
}
