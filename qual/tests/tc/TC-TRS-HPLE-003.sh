tc_TRS_HPLE_003() {
    local F="$1"; local B="$F/TC-TRS-HPLE-003"
    local out

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has() { printf '%s' "$1" | grep -qF "$2"; }

    # 1. a genuinely open, selected cross-tier parameter validates cleanly
    _scn "a genuinely open, selected cross-tier parameter validates cleanly"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>&1 || true)
    if _has "$out" 'E519' || _has "$out" 'E523'; then
        fail "unexpected cross-tier binding error on a genuinely open, selected parameter"
    else
        pass "no cross-tier binding error raised"
    fi

    # 2. binding a parameter of a feature the owning tier does not select is rejected
    _scn "binding a parameter of a feature the owning tier does not select is rejected"
    out=$("$SYSCRIBE" -m "$B/e519" validate 2>&1 || true)
    _has "$out" 'E519' && pass "E519 raised for a not-selected-by-owner target" \
        || fail "E519 not raised for a not-selected-by-owner target"

    # 3. double-binding a parameter a nearer tier already closed is rejected
    _scn "double-binding a parameter a nearer tier already closed is rejected"
    out=$("$SYSCRIBE" -m "$B/e523" validate 2>&1 || true)
    if _has "$out" 'E523' && _has "$out" 'CONF-PEER-BOUND-001'; then
        pass "E523 raised, naming the nearer peer Configuration that already bound it"
    else
        fail "E523 not raised (or did not name the nearer peer Configuration)"
    fi
}
