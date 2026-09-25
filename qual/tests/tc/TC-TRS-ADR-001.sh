tc_TRS_ADR_001() {
    local F="$1"; local B="$F/TC-TRS-ADR-001"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out show w047

    out=$("$SYSCRIBE" -m "$B/model" validate 2>/dev/null) || true
    w047=$(grep -F "| W047 |" <<<"$out" || true)

    _scn "deciders on an ADR is a recognized field"
    grep -qF "ADR-DEC-001.md" <<<"$w047" \
        && fail "W047 on the ADR's deciders: field" \
        || pass "no W047 on ADR-DEC-001.md"
    grep -qF "Jane Doe" <<<"$out" \
        && fail "a finding names the free-text decider" \
        || pass "the free-text decider is not resolved as a reference"

    _scn "show displays the deciders and the date"
    show=$("$SYSCRIBE" -m "$B/model" show ADR-DEC-001 2>/dev/null) || true
    grep -F "**deciders**" <<<"$show" | grep -F "Stakeholders::SystemsEngineer" | grep -qF "Jane Doe" \
        && pass "show lists both deciders" \
        || fail "show has no deciders row listing both entries"
    grep -F "**date**" <<<"$show" | grep -qF "2026-05-20" \
        && pass "show lists the ADR date" \
        || fail "show has no date row"

    _scn "deciders on a non-ADR element is still unrecognized"
    grep -F "SystemsEngineer.md" <<<"$w047" | grep -qF "'deciders'" \
        && pass "W047 names deciders on the PartDef" \
        || fail "no W047 for deciders on a PartDef"
}
