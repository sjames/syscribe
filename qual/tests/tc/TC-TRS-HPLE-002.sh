tc_TRS_HPLE_002() {
    local F="$1"; local B="$F/TC-TRS-HPLE-002"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has() { printf '%s' "$1" | grep -qF "$2"; }

    # 1. a dotted key resolves a peer feature's parameter reachable through subConfigurations
    _scn "a dotted key resolves a peer feature's parameter reachable through subConfigurations"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>&1 || true)
    _has "$out" 'E222' \
        && fail "unexpected E222 for a parameter genuinely reachable through subConfigurations" \
        || pass "no unresolved-reference error raised"

    # 2. a dotted key naming a FeatureDef unreachable by any means is still rejected
    _scn "a dotted key naming a FeatureDef unreachable by any means is still rejected"
    out=$("$SYSCRIBE" -m "$B/e222" validate 2>&1 || true)
    _has "$out" 'E222' && pass "E222 raised for an unreachable FeatureDef" \
        || fail "E222 not raised for an unreachable FeatureDef"
}
