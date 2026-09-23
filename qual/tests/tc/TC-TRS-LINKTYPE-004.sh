tc_TRS_LINKTYPE_004() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-004"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)
    _scn "within bounds validates cleanly"
    has "$out" "REQ-LT4-011.md" "E635|W631" && fail "unexpected cardinality finding" || pass "REQ-LT4-011 clean"
    _scn "more targets than the upper bound raises E635"
    has "$out" "REQ-LT4-010.md" "E635" && pass "E635 raised" || fail "E635 not raised"
    _scn "a non-draft in-scope element under the lower bound raises W631"
    has "$out" "REQ-LT4-020.md" "W631" && pass "W631 raised" || fail "W631 not raised"
    _scn "a draft element under the lower bound raises nothing"
    has "$out" "REQ-LT4-021.md" "W631" && fail "W631 raised on draft" || pass "draft suppressed"
}
