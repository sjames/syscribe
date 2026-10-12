tc_TRS_MODELS_001() {
    local F="$1"
    local G="$F/TC-TRS-MODELS-001/good"
    local B="$F/TC-TRS-MODELS-001/bad"

    SCENARIO_NAME="a model whose executable parts model the architecture consistently is clean"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" validate 2>/dev/null || true)
    assert_no_code "E990"
    assert_no_code "E991"
    assert_no_code "W990"
    assert_no_code "W991"
    assert_no_code "W992"
    assert_no_code "W047"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$G" links Exec::Monitor 2>/dev/null || true)
    assert_output_contains "| models | Arch::Manager | PartDef |"

    SCENARIO_NAME="each correspondence defect is reported"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B" validate 2>/dev/null || true)
    assert_has_code "E990"
    assert_output_contains "\`models: Arch::Manger\` does not resolve"
    assert_count "E991" 2
    assert_output_contains "port 'debug' corresponds to no port of 'Arch::Manager'"
    assert_output_contains "port 'cmd' is \`out\` but its counterpart 'Arch::Relay.cmd' is \`in\`"
    assert_has_code "W990"
    assert_output_contains "port 'profile' of 'Arch::Manager' has no counterpart"
    assert_has_code "W991"
    assert_has_code "W992"
    assert_output_contains "TC-QM-001 verifies REQ-QM-002"
}
