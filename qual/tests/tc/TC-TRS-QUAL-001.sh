tc_TRS_QUAL_001() {
    local Q; Q="$(cd "$1/.." && pwd)"   # $1 is qual/fixtures → the qualification model root

    SCENARIO_NAME="the qualification model validates clean of W047"; _SCEN_PASS=0; _SCEN_FAIL=0
    printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$Q" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_zero
    assert_no_code "W047"

    run_scenario_static() {
        _flush_scenario
        SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0
        printf "  ▶ %s\n" "$SCENARIO_NAME"
    }
    run_scenario_static "the TVR version is not maintained in model data"
    local fm; fm=$(awk 'NR==1 && /^---$/ {next} /^---$/ {exit} {print}' "$Q/_index.md" || true)
    grep -qE '^version:' <<<"$fm" \
        && fail "qual/_index.md still carries a top-level version: key" \
        || pass "qual/_index.md has no top-level version: key"
    grep -qF 'generate_tvr.sh" "$SYSCRIBE_VERSION"' "$Q/tests/run_qual.sh" \
        && grep -qF 'SYSCRIBE_VERSION=$("$SYSCRIBE" --version' "$Q/tests/run_qual.sh" \
        && pass "run_qual.sh feeds the binary's --version to generate_tvr.sh" \
        || fail "run_qual.sh does not pass the binary's --version to the TVR generator"
}
