tc_TRS_FMEA_004() {
    local F="$1"; local M="$F/TC-TRS-FMEA-004/model"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out rep
    out=$("$SYSCRIBE" -m "$M" validate 2>/dev/null) || true

    _scn "a row without id raises E923 naming the row"
    local e923; e923=$(grep -F "| E923 |" <<<"$out" || true)
    if [ -n "$e923" ]; then pass "E923 raised"; else fail "E923 not raised"; fi
    grep -qF "FMEA-ROWS-001" <<<"$e923" && pass "E923 is on the sheet" || fail "E923 not attributed to the sheet"
    grep -qE "row 2\b" <<<"$e923" && pass "E923 names row 2" || fail "E923 does not name row 2: $e923"
    grep -qF "Orphan row with no id" <<<"$e923" && pass "E923 names the failure mode" || fail "E923 lacks the failure mode"

    _scn "an explicit rpn that disagrees with S×O×D raises W928"
    local w928; w928=$(grep -F "| W928 |" <<<"$out" || true)
    grep -qF "FM-ROWS-003" <<<"$w928" && pass "W928 names FM-ROWS-003" || fail "W928 missing for FM-ROWS-003"
    grep -qF "100" <<<"$w928" && grep -qF "60" <<<"$w928" \
        && pass "W928 names explicit 100 and computed 60" || fail "W928 does not name both values: $w928"
    rep=$("$SYSCRIBE" -m "$M" fmea report --json 2>/dev/null) || true
    local rpn; rpn=$(jq -r '[.. | objects | select((.id? // "") == "FM-ROWS-003") | .rpn] | first // empty' <<<"$rep" 2>/dev/null || true)
    [ "$rpn" = "60" ] && pass "fmea report keeps computed RPN 60" || fail "fmea report RPN for FM-ROWS-003 is '$rpn' (expected 60)"

    _scn "a consistent or partial rpn raises no W928"
    grep -qF "FM-ROWS-001" <<<"$w928" && fail "W928 wrongly fired for consistent FM-ROWS-001" || pass "no W928 for FM-ROWS-001"
    grep -qF "FM-ROWS-004" <<<"$w928" && fail "W928 wrongly fired for partial FM-ROWS-004" || pass "no W928 for FM-ROWS-004"
}
