tc_TRS_FMA_010() {
    local F="$1"; local V="$F/TC-TRS-FMA-005/void"

    SCENARIO_NAME="void model reports minimal correction sets"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local j; j=$("$SYSCRIBE" -m "$V" feature-check --deep --json 2>/dev/null || true)
    local n; n=$(printf '%s' "$j" | jq -r '.diagnoses | length' 2>/dev/null || true)
    [ "${n:-0}" -ge 1 ] && pass "at least one correction set reported (${n})" || fail "no diagnoses reported"

    SCENARIO_NAME="a correction set names the requires or excludes constraint"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$j" | jq -e '[.diagnoses[][]] | map(ascii_downcase) | any(test("excludes|requires"))' >/dev/null 2>&1 \
        && pass "a diagnosis points at the relaxable constraint" || fail "no diagnosis names requires/excludes"
}
