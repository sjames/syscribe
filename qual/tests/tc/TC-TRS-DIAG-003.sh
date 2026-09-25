tc_TRS_DIAG_003() {
    local F="$1"; local B="$F/TC-TRS-DIAG-003"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out ids

    out=$("$SYSCRIBE" -m "$B/model" validate 2>/dev/null) || true
    ids=$(grep -E "\| W40[67] \|" <<<"$out" || true)

    _scn "a PlantUML companion diagram has no inline SVG to check"
    grep -qF "PumlCompanion.md" <<<"$ids" \
        && fail "W406/W407 on the pumlMode: companion diagram" \
        || pass "no W406/W407 on PumlCompanion.md"

    _scn "a structured layout diagram has no inline SVG to check"
    grep -qF "Structured.md" <<<"$ids" \
        && fail "W406/W407 on the layout: diagram" \
        || pass "no W406/W407 on Structured.md"

    _scn "an inline SVG diagram is still checked"
    local inl; inl=$(grep -F "| W406 |" <<<"$ids" | grep -F "InlineSvg.md" || true)
    grep -qF "'s-inline-missing'" <<<"$inl" \
        && pass "W406 names s-inline-missing on InlineSvg.md" \
        || fail "W406 missing for the inline SVG diagram"
    grep -qF "'s-vehicle'" <<<"$inl" \
        && fail "W406 wrongly names s-vehicle, which the SVG carries" \
        || pass "no W406 for the id present in the SVG"
}
