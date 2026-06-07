tc_TRS_FMA_005() {
    local F="$1"
    local V="$F/TC-TRS-FMA-005/void"
    local VF="$F/TC-TRS-FMA-005/void-fixed"

    SCENARIO_NAME="void explanation names the conflicting constraints"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local line; line=$("$SYSCRIBE" -m "$V" feature-check --deep 2>/dev/null | grep -F "| E223 |" || true)
    local ok=1
    for needle in "Features::A" "Features::B" "requires" "excludes"; do
        printf '%s' "$line" | grep -qF "$needle" || ok=0
    done
    [ "$ok" -eq 1 ] && pass "E223 explanation names A, B, requires, excludes" || fail "explanation incomplete: $line"

    SCENARIO_NAME="soundness: removing the excludes un-voids the model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local v; v=$("$SYSCRIBE" -m "$VF" feature-check --deep --json 2>/dev/null | jq -r '.void' 2>/dev/null || true)
    [ "$v" = "false" ] && pass "void-fixed model is not void" || fail "void-fixed still void (v=$v)"
}
