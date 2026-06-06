tc_TRS_FM_001() {
    local F="$1"; local B="$F/TC-TRS-FM-001"

    # Scenario: feature-check is discoverable in help.
    SCENARIO_NAME="feature-check is listed in help"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" --help 2>/dev/null | grep -qE "^\s*feature-check" \
        && pass "feature-check listed in --help" || fail "feature-check missing from --help"

    # Scenario: a clean feature model exits 0.
    SCENARIO_NAME="clean feature model exits 0"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$B/clean" feature-check >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "clean exits 0" || fail "clean exit ${ec} (expected 0)"

    # Scenario: a feature model with a violation exits 1 (reuse FM-002 violations).
    SCENARIO_NAME="violations exit 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$F/TC-TRS-FM-002/violations" feature-check >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 1 ] && pass "violations exit 1" || fail "violations exit ${ec} (expected 1)"

    # Scenario: no feature model is dormant (notice + exit 0).
    SCENARIO_NAME="no feature model is dormant"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$B/no-fm" feature-check 2>/dev/null || true)
    printf '%s' "$out" | grep -qiF "no feature model" \
        && pass "prints no-feature-model notice" || fail "missing dormancy notice"
    "$SYSCRIBE" -m "$B/no-fm" feature-check >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "dormant exits 0" || fail "dormant exit ${ec} (expected 0)"

    # Scenario: --json emits a JSON array of findings.
    SCENARIO_NAME="--json emits structured findings"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$F/TC-TRS-FM-002/violations" feature-check --json 2>/dev/null || true)
    local n; n=$(printf '%s' "$j" | jq -r 'length' 2>/dev/null || true)
    [ "${n:-0}" -ge 1 ] && pass "--json is an array with $n finding(s)" || fail "--json not a non-empty array (got '${n}')"
}
