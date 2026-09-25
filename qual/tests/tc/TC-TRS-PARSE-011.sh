tc_TRS_PARSE_011() {
    local F="$1"; local B="$F/TC-TRS-PARSE-011"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out list show e027 w052 t

    out=$("$SYSCRIBE" -m "$B/notes" validate 2>/dev/null) || true

    _scn "an about comment does not become an element"
    grep -qF "SafetyNote.md" <<<"$out" \
        && fail "a finding names SafetyNote.md: $(grep -F "SafetyNote.md" <<<"$out" | head -1 | cut -c1-160)" \
        || pass "no finding names SafetyNote.md"
    list=$("$SYSCRIBE" -m "$B/notes" ls VehicleSystem 2>/dev/null) || true
    grep -qF "VehicleSystem::Engine |" <<<"$list" \
        && pass "ls lists VehicleSystem::Engine" || fail "ls does not list VehicleSystem::Engine"
    grep -qF "VehicleSystem::SafetyNote" <<<"$list" \
        && fail "the comment file is listed as an element" \
        || pass "no element VehicleSystem::SafetyNote"

    _scn "the comment is shown on every listed element"
    for t in VehicleSystem::Engine VehicleSystem::Transmission; do
        show=$("$SYSCRIBE" -m "$B/notes" show "$t" 2>/dev/null) || true
        grep -qF "## Note: SafetyNote" <<<"$show" && grep -qF "safety analysis per ISO 26262" <<<"$show" \
            && pass "$t shows the SafetyNote comment" \
            || fail "$t does not show the SafetyNote comment"
    done
    show=$("$SYSCRIBE" -m "$B/notes" show REQ-VS-001 2>/dev/null) || true
    grep -qF "## Note: ColdStartNote" <<<"$show" && grep -qF "Cold-start behaviour" <<<"$show" \
        && pass "a comment listing a stable id attaches to the requirement" \
        || fail "REQ-VS-001 does not show the ColdStartNote comment"

    out=$("$SYSCRIBE" -m "$B/bad" validate 2>/dev/null) || true
    e027=$(grep -F "| E027 |" <<<"$out" || true)
    w052=$(grep -F "| W052 |" <<<"$out" || true)

    _scn "an unresolved entry raises E027"
    grep -F "PartialNote.md" <<<"$e027" | grep -qF "VehicleSystem::Ghost" \
        && pass "E027 names PartialNote.md and VehicleSystem::Ghost" \
        || fail "no E027 for PartialNote.md's VehicleSystem::Ghost"
    show=$("$SYSCRIBE" -m "$B/bad" show VehicleSystem::Engine 2>/dev/null) || true
    grep -qF "## Note: PartialNote" <<<"$show" \
        && pass "the partial comment still attaches to VehicleSystem::Engine" \
        || fail "the partial comment is not shown on VehicleSystem::Engine"
    grep -F "OrphanNote.md" <<<"$e027" | grep -qF "VehicleSystem::Nowhere" \
        && pass "E027 names OrphanNote.md and VehicleSystem::Nowhere" \
        || fail "no E027 for OrphanNote.md"
    list=$("$SYSCRIBE" -m "$B/bad" ls VehicleSystem 2>/dev/null) || true
    grep -qF "VehicleSystem::OrphanNote" <<<"$list" \
        && pass "the unattachable comment is kept as its own element" \
        || fail "the unattachable comment was dropped"

    _scn "ignored fields raise W052"
    grep -F "StructNote.md" <<<"$w052" | grep -qF "supertype" \
        && pass "W052 names StructNote.md's supertype field" \
        || fail "no W052 for StructNote.md's supertype"
    grep -F "VehicleSystem/_index.md" <<<"$w052" | grep -qF "about" \
        && pass "W052 names about: on the _index.md" \
        || fail "no W052 for about: on the _index.md"
    list=$("$SYSCRIBE" -m "$B/bad" ls VehicleSystem 2>/dev/null) || true
    grep -qF "VehicleSystem::StructNote" <<<"$list" \
        && fail "the comment with an ignored field became an element" \
        || pass "the comment with an ignored field is still a comment, not an element"
}
