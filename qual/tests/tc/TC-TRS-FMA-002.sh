tc_TRS_FMA_002() {
    local F="$1"
    local V="$F/TC-TRS-FMA-005/void"
    local VF="$F/TC-TRS-FMA-005/void-fixed"
    local NOFM="$F/TC-TRS-FM-001/no-fm"

    SCENARIO_NAME="--deep is listed in help"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" --help 2>/dev/null | grep -q -- "--deep" \
        && pass "--deep documented in help" || fail "--deep missing from help"

    SCENARIO_NAME="--deep is opt-in (no E223 without it)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local base; base=$("$SYSCRIBE" -m "$V" feature-check 2>/dev/null || true)
    printf '%s' "$base" | grep -qF "| E223 |" && fail "E223 emitted without --deep" || pass "no E223 without --deep"
    "$SYSCRIBE" -m "$V" feature-check >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "base feature-check on void model exits 0" || fail "base exit ${ec} (expected 0)"

    SCENARIO_NAME="exit codes: sound 0, void 1"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$VF" feature-check --deep >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "sound model --deep exits 0" || fail "sound exit ${ec} (expected 0)"
    "$SYSCRIBE" -m "$V" feature-check --deep >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 1 ] && pass "void model --deep exits 1" || fail "void exit ${ec} (expected 1)"

    SCENARIO_NAME="--json carries the documented keys"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$VF" feature-check --deep --json 2>/dev/null || true)
    printf '%s' "$j" | jq -e 'has("void") and has("deadFeatures") and has("coreFeatures") and has("falseOptionalFeatures") and has("invalidConfigurations") and has("findings") and has("schemaVersion")' >/dev/null 2>&1 \
        && pass "all deep --json keys present" || fail "missing deep --json keys"

    SCENARIO_NAME="dormant with no feature model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$NOFM" feature-check --deep 2>/dev/null || true)
    printf '%s' "$out" | grep -qiF "no feature model" && pass "dormant notice" || fail "missing dormant notice"
    "$SYSCRIBE" -m "$NOFM" feature-check --deep >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "dormant exits 0" || fail "dormant exit ${ec}"
}
