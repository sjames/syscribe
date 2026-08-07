tc_TRS_HPLE_004() {
    local F="$1"; local B="$F/TC-TRS-HPLE-004"
    local out; local rc

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _has() { printf '%s' "$1" | grep -qF "$2"; }

    # 1. a still-open required parameter reachable through subConfigurations is a warning
    _scn "a still-open required parameter reachable through subConfigurations is a warning"
    out=$("$SYSCRIBE" -m "$B/w513" validate 2>&1) && rc=0 || rc=$?
    if _has "$out" 'W513' && _has "$out" 'capacityKg'; then
        pass "W513 raised naming capacityKg"
    else
        fail "W513 not raised for the still-open parameter"
    fi
    [ "$rc" -eq 0 ] && pass "exit code stays zero without --deny" \
        || fail "exit code was $rc without --deny (expected 0)"

    # 2. the same warning is gateable via --deny
    _scn "the same warning is gateable via --deny"
    "$SYSCRIBE" -m "$B/w513" validate --deny W513 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -ne 0 ] && pass "exit code is non-zero under --deny W513 (got $rc)" \
        || fail "exit code stayed zero under --deny W513"

    # 3. a fully-closed subtree raises no such warning
    _scn "a fully-closed subtree raises no such warning"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>&1 || true)
    _has "$out" 'W513' \
        && fail "unexpected W513 on a fully-closed consolidated subtree" \
        || pass "no W513 raised"
}
