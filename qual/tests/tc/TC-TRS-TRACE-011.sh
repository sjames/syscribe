tc_TRS_TRACE_011() {
    local F="$1"; local B="$F/TC-TRS-TRACE-011"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out hits

    out=$("$SYSCRIBE" -m "$B/model" validate 2>/dev/null) || true
    hits=$(grep -F "| W005 |" <<<"$out" || true)

    _scn "a requirement derived only from a SafetyGoal is not an orphan"
    grep -qF "REQ-OR-001" <<<"$hits" && fail "W005 on REQ-OR-001 (derivedFromSafetyGoal)" \
        || pass "no W005 on REQ-OR-001"

    _scn "a requirement derived only from a CybersecurityGoal is not an orphan"
    grep -qF "REQ-OR-002" <<<"$hits" && fail "W005 on REQ-OR-002 (derivedFromCybersecurityGoal)" \
        || pass "no W005 on REQ-OR-002"
    grep -qF "REQ-OR-003" <<<"$hits" && fail "W005 on REQ-OR-003 (legacy derivedFromSecurityGoal)" \
        || pass "no W005 on REQ-OR-003 (legacy key)"

    _scn "a requirement with no upstream link is still an orphan"
    grep -qF "REQ-OR-004" <<<"$hits" && pass "W005 on REQ-OR-004" \
        || fail "W005 missing on the unlinked REQ-OR-004"
    local n; n=$(grep -cF "| W005 |" <<<"$out" || true)
    [ "${n:-0}" -eq 1 ] && pass "exactly one W005" || fail "W005 count = ${n:-0} (expected 1)"
}
