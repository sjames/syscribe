tc_TRS_FMA_006() {
    local F="$1"
    local A="$F/TC-TRS-FMA-003/anomalies"
    local BIG="$F/TC-TRS-FMA-006/big"

    SCENARIO_NAME="determinism: two runs are identical"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local a b
    a=$("$SYSCRIBE" -m "$A" feature-check --deep --json 2>/dev/null || true)
    b=$("$SYSCRIBE" -m "$A" feature-check --deep --json 2>/dev/null || true)
    [ "$a" = "$b" ] && pass "deep --json output is identical across runs" || fail "non-deterministic output"

    SCENARIO_NAME="size guard skips gracefully"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local out; out=$("$SYSCRIBE" -m "$BIG" feature-check --deep 2>/dev/null || true)
    printf '%s' "$out" | grep -qiF "skip" && pass "prints skip diagnostic" || fail "no skip diagnostic"
    "$SYSCRIBE" -m "$BIG" feature-check --deep >/dev/null 2>&1 && ec=0 || ec=$?
    [ "${ec:-0}" -eq 0 ] && pass "skipped deep analysis exits 0 (not a hang/false-OK)" || fail "guard exit ${ec}"

    SCENARIO_NAME="scope statement: Boolean layer only"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local scope; scope=$("$SYSCRIBE" -m "$A" feature-check --deep 2>/dev/null || true)
    printf '%s' "$scope" | grep -qiF "Boolean feature layer only" \
        && pass "output states Boolean-only scope" || fail "missing Boolean-only scope statement"
}
