tc_TRS_HPLE_001() {
    local F="$1"; local B="$F/TC-TRS-HPLE-001"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has() { printf '%s' "$1" | grep -qF "$2"; }

    # 1. a valid peer Configuration consolidates cleanly
    _scn "a valid peer Configuration consolidates cleanly"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>&1 || true)
    if _has "$out" 'E516' || _has "$out" 'E517' || _has "$out" 'E518'; then
        fail "unexpected subConfigurations error on a valid peer consolidation"
    else
        pass "no subConfigurations error raised"
    fi

    # 2. a dangling subConfigurations name is rejected
    _scn "a dangling subConfigurations name is rejected"
    out=$("$SYSCRIBE" -m "$B/e516" validate 2>&1 || true)
    _has "$out" 'E516' && pass "E516 raised for a dangling subConfigurations name" \
        || fail "E516 not raised for a dangling subConfigurations name"

    # 3. a subConfigurations name resolving to a non-Configuration is rejected
    _scn "a subConfigurations name resolving to a non-Configuration is rejected"
    out=$("$SYSCRIBE" -m "$B/e517" validate 2>&1 || true)
    _has "$out" 'E517' && pass "E517 raised for a non-Configuration target" \
        || fail "E517 not raised for a non-Configuration target"

    # 4. a subConfigurations name resolving to a SAT-invalid Configuration is rejected
    _scn "a subConfigurations name resolving to a SAT-invalid Configuration is rejected"
    out=$("$SYSCRIBE" -m "$B/e518" validate 2>&1 || true)
    _has "$out" 'E518' && pass "E518 raised for a SAT-invalid peer Configuration" \
        || fail "E518 not raised for a SAT-invalid peer Configuration"
}
