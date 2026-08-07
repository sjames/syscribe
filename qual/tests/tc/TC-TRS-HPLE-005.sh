tc_TRS_HPLE_005() {
    local F="$1"; local B="$F/TC-TRS-HPLE-005"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has() { printf '%s' "$1" | grep -qF "$2"; }

    # 1. bindTo resolves normally within its own model (positive control)
    _scn "bindTo resolves normally within its own model (positive control)"
    out=$("$SYSCRIBE" -m "$B/lower_selfbound/model" feature-check 2>&1 || true)
    _has "$out" 'E202' && pass "E202 raised when the match is genuinely local" \
        || fail "E202 not raised for a genuinely local bindTo match"

    # 2. a lower tier's bindTo target never becomes visible to a separate higher tier
    _scn "a lower tier's bindTo target never becomes visible to a separate higher tier"
    out=$("$SYSCRIBE" -m "$B/higher/model" feature-check 2>&1 || true)
    _has "$out" 'E202' \
        && fail "unexpected E202 on the higher tier — the lower tier's bindTo/range leaked across the repo boundary" \
        || pass "no E202 raised on the higher tier's own feature-check"

    # 3. a higher tier's binding never leaks down into the lower tier's own validation
    _scn "a higher tier's binding never leaks down into the lower tier's own validation"
    out=$("$SYSCRIBE" -m "$B/lower/model" feature-check 2>&1 || true)
    _has "$out" 'E202' \
        && fail "unexpected E202 on the lower tier — a sibling higher-tier model's binding leaked down" \
        || pass "no E202 raised on the lower tier's own feature-check"
}
