tc_TRS_FM_007() {
    local F="$1"

    SCENARIO_NAME="mandatory: true + groupKind: optional selected everywhere raises no W012"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/TC-TRS-FM-007/mand" feature-check 2>/dev/null || true)
    assert_no_code "W012"

    SCENARIO_NAME="optional feature selected everywhere still raises W012 suggesting mandatory: true"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$F/TC-TRS-FM-007/opt" feature-check 2>/dev/null || true)
    assert_has_code "W012"
    assert_output_contains "consider mandatory: true"

    SCENARIO_NAME="grid-tie model spells its mandatory groups mandatory: true and has no feature-check findings"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local gti; gti="$(cd "$F/../.." && pwd)/model_gti"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$gti" feature-check 2>/dev/null || true)
    assert_no_code "W012"
    assert_output_contains "Feature model OK — 0 findings."
}
