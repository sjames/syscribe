tc_TRS_PARSE_010() {
    local F="$1"; local B="$F/TC-TRS-PARSE-010"
    _scn() { _flush_scenario; SCENARIO_NAME="$1"; _SCEN_PASS=0; _SCEN_FAIL=0; printf "  ▶ %s\n" "$1"; }
    local out list show w051 e026

    out=$("$SYSCRIBE" -m "$B/variants" validate 2>/dev/null) || true

    _scn "a variant file does not become an element"
    grep -qF "| W042 |" <<<"$out" \
        && fail "W042 raised: $(grep -F "| W042 |" <<<"$out" | head -1 | cut -c1-160)" \
        || pass "no W042"
    list=$("$SYSCRIBE" -m "$B/variants" ls VehicleSystem 2>/dev/null) || true
    grep -qF "VehicleSystem::Engine |" <<<"$list" \
        && pass "ls lists the base element VehicleSystem::Engine" \
        || fail "ls does not list VehicleSystem::Engine"
    grep -qF "VehicleSystem::Engine." <<<"$list" \
        && fail "a variant file is listed as an element: $(grep -F "VehicleSystem::Engine." <<<"$list" | head -1)" \
        || pass "no variant file (VehicleSystem::Engine.<suffix>) is an element"

    _scn "the variant bodies are shown on the base element"
    show=$("$SYSCRIBE" -m "$B/variants" show VehicleSystem::Engine 2>/dev/null) || true
    grep -qF "## Documentation (de)" <<<"$show" && grep -qF "Der Motor wandelt" <<<"$show" \
        && pass "German documentation shown" \
        || fail "no Documentation (de) section with the German body"
    grep -qF "## Documentation (fr)" <<<"$show" && grep -qF "Le moteur convertit" <<<"$show" \
        && pass "French documentation shown" \
        || fail "no Documentation (fr) section with the French body"

    _scn "a duplicate locale and a structural field raise W051"
    w051=$(grep -F "| W051 |" <<<"$out" || true)
    grep -qF "Engine.zz_de_copy.md" <<<"$w051" \
        && pass "W051 names the duplicate de variant" \
        || fail "no W051 for the duplicate de variant"
    grep -F "Engine.it.md" <<<"$w051" | grep -qF "isAbstract" \
        && pass "W051 names the ignored isAbstract field" \
        || fail "no W051 for the variant's structural field"
    grep -qF "Zweite deutsche Fassung" <<<"$show" \
        && fail "the duplicate variant's body replaced the first" \
        || pass "the first de variant wins"

    _scn "a variant naming a missing element raises E026"
    out=$("$SYSCRIBE" -m "$B/dangling" validate 2>/dev/null) || true
    e026=$(grep -F "| E026 |" <<<"$out" || true)
    grep -F "Ghost_de.md" <<<"$e026" | grep -qF "VehicleSystem::Ghost" \
        && pass "E026 names the dangling variant and its target" \
        || fail "no E026 for the dangling variant"
}
