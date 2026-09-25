tc_TRS_ALLOC_003() {
    local F="$1"; local B="$F/TC-TRS-ALLOC-003"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits mx

    _scn "a features-form allocation on a PartDef raises W930"
    out=$("$SYSCRIBE" -m "$B/misplaced" validate 2>/dev/null) || true
    hits=$(grep -F "| W930 |" <<<"$out" || true)
    grep -qF "Controller.md" <<<"$hits" && grep -qF "'thrustAlloc'" <<<"$hits" \
        && pass "W930 names the entry on Controller" || fail "W930 missing for the misplaced entry: $hits"
    mx=$("$SYSCRIBE" -m "$B/misplaced" matrix --allocations 2>/dev/null) || true
    grep -qF "no allocation edges" <<<"$mx" && pass "no allocation edge from the misplaced entry" \
        || fail "the misplaced entry produced an edge: $mx"

    _scn "the same entry on an Allocation element is an edge"
    out=$("$SYSCRIBE" -m "$B/clean" validate 2>/dev/null) || true
    grep -qF "| W930 |" <<<"$out" && fail "W930 raised on a type: Allocation element" || pass "no W930"
    mx=$("$SYSCRIBE" -m "$B/clean" matrix --allocations 2>/dev/null) || true
    grep -qE "ComputeThrust +\| +✓" <<<"$mx" && pass "edge ComputeThrust -> Board shown" \
        || fail "edge missing from matrix --allocations: $mx"
}
