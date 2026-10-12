tc_TRS_PROJ_009() {
    local F="$1"; local B="$F/TC-TRS-PROJ-009"

    SCENARIO_NAME="a parameter with no default is one W248, not a parse error"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/nodefault" validate 2>/dev/null || true)
    assert_count "W248" 1
    assert_output_contains "parameter 'Features::Profile.limit' has no value or default"
    assert_output_contains "in M, N"
    assert_output_contains "validate --config"
    assert_no_code "E882"
    assert_no_code "E884"
    assert_no_code "W933"

    SCENARIO_NAME="--config checks the machine at the bound value"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/nodefault" validate --config CONF-P9-AA-001 2>/dev/null || true)
    assert_no_code "W248"
    assert_no_code "W933"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/badbinding" validate --config CONF-P9-AA-001 2>/dev/null || true)
    assert_has_code "W933"
    assert_output_contains "uses 'ten'"

    SCENARIO_NAME="--all-configs checks each Configuration at its own bindings"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/twoconfigs" validate --all-configs 2>/dev/null || true)
    assert_output_contains "| CONF-P9-AA-001 | 0 | 2 | pass |"
    assert_output_contains "| CONF-P9-BB-001 | 0 | 4 | pass |"

    SCENARIO_NAME="a parameter with a default keeps today's behaviour"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$B/default" validate 2>/dev/null || true)
    assert_no_code "W248"
    assert_no_code "E882"
    assert_no_code "E884"
    assert_no_code "W933"
}
