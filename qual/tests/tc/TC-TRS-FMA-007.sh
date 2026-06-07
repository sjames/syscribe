tc_TRS_FMA_007() {
    local F="$1"; local N="$F/TC-TRS-FMA-007/void-with-noise"

    SCENARIO_NAME="explanation is a minimal conflict set (excludes unrelated features)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local line; line=$("$SYSCRIBE" -m "$N" feature-check --deep 2>/dev/null | grep -F "| E223 |" || true)
    local ok=1
    for needle in "Features::A" "Features::B" "requires" "excludes"; do
        printf '%s' "$line" | grep -qF "$needle" || ok=0
    done
    [ "$ok" -eq 1 ] && pass "core names the real conflict (A, B, requires, excludes)" || fail "core missing conflict terms: $line"
    # Minimality: the unrelated features N1/N2/N3 must NOT appear in the core.
    printf '%s' "$line" | grep -qE "Features::N[123]" \
        && fail "core wrongly includes unrelated features: $line" || pass "core excludes unrelated features (minimal)"
}
