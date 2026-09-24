tc_TRS_LINKTYPE_001() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-001"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    _scn "a well-formed [linkTypes] table validates cleanly"
    out=$("$SYSCRIBE" -m "$FX/good" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'W63[01]|E63[0-9]' && fail "unexpected link-type finding on good fixture" || pass "good fixture clean"

    _scn "each structural defect raises W630 and the entry is ignored"
    out=$("$SYSCRIBE" -m "$FX/bad" validate 2>&1 || true)
    for n in Bad-Name satisfies alpha badCard lowerNoSource unknownType badBase relaxNoExtends badRelax dupInverse; do
        printf '%s' "$out" | grep 'W630' | grep -qF -- "$n" && pass "W630 names $n" || fail "W630 does not name $n"
    done
    has "$out" "UsesIgnored.md" "E630" && pass "use of ignored entry raises E630" || fail "no E630 for use of ignored entry"

    _scn "an unknown key raises W630 but the entry stays usable"
    printf '%s' "$out" | grep 'W630' | grep -q 'colour' && pass "W630 names unknown key" || fail "no W630 for unknown key"
    has "$out" "UsesExtraKey.md" "E630" && fail "entry with unknown key was ignored" || pass "entry with unknown key still usable"

    _scn "a model without [linkTypes] is unaffected"
    out=$("$SYSCRIBE" -m "$FX/none" validate 2>&1 || true)
    printf '%s' "$out" | grep -qE 'W63[01]|E63[0-9]' && fail "finding on unconfigured model" || pass "unconfigured model unaffected"
}
