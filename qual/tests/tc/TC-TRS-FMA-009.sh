tc_TRS_FMA_009() {
    local F="$1"; local M="$F/TC-TRS-FMA-009/model"

    # The realistic model (XOR Platform, OR Conn, cross-tree requires/excludes)
    # has exactly 10 valid configurations (see fixture derivation).
    SCENARIO_NAME="count of valid configurations is 10"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$M" feature-check --count --json 2>/dev/null || true)
    local n; n=$(printf '%s' "$j" | jq -r '.variantCount' 2>/dev/null || true)
    [ "$n" = "10" ] && pass "variantCount = 10" || fail "variantCount = '${n}' (expected 10)"

    SCENARIO_NAME="enumeration lists each valid configuration"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local je; je=$("$SYSCRIBE" -m "$M" feature-check --enumerate --json 2>/dev/null || true)
    local c; c=$(printf '%s' "$je" | jq -r '.variants | length' 2>/dev/null || true)
    [ "$c" = "10" ] && pass "enumerated 10 configurations" || fail "enumerated '${c}' (expected 10)"
    # determinism of enumeration
    local je2; je2=$("$SYSCRIBE" -m "$M" feature-check --enumerate --json 2>/dev/null || true)
    [ "$je" = "$je2" ] && pass "enumeration is deterministic" || fail "enumeration not deterministic"
}
