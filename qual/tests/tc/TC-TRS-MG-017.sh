tc_TRS_MG_017() {
    local F="$1"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # ── allocation: sources × targets ✓ matrix + gap rollup (TC-TRS-MG-006/flat) ──
    out=$("$SYSCRIBE" -m "$F/TC-TRS-MG-006/flat" matrix --allocations 2>/dev/null || true)

    _scn "matrix --allocations renders a sources × targets matrix with ✓ cells"
    { printf '%s' "$out" | grep -qiF "sources" \
        && printf '%s' "$out" | grep -qF "✓" \
        && printf '%s' "$out" | grep -qE '^\s*ActA\s*\|'; } \
        && pass "allocation ✓ matrix rendered" || fail "no sources×targets ✓ matrix"

    _scn "an unallocated source is reported as a gap"
    printf '%s' "$out" | grep -qF "Unallocated sources" \
        && printf '%s' "$out" | grep -F "Unallocated sources" | grep -qF "ActC" \
        && pass "gap (ActC) reported in the rollup" || fail "unallocated source not reported"

    # ── trade-study: Configuration × MoE matrix with a winner (TC-TRS-MG-012/exact) ──
    out=$("$SYSCRIBE" -m "$F/TC-TRS-MG-012/exact" trade-study 2>/dev/null || true)

    _scn "trade-study renders a MoE × Configuration score matrix"
    { printf '%s' "$out" | grep -qiF "MoE" \
        && printf '%s' "$out" | grep -qiF "Conf"; } \
        && pass "MoE × Configuration matrix rendered" || fail "no trade-study score matrix"

    _scn "the winning configuration is marked"
    printf '%s' "$out" | grep -qiF "WINNER" \
        && pass "winner highlighted" || fail "no winner marked"
}
