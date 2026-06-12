tc_TRS_TAG_002() {
    local F="$1"; local M="$F/TC-TRS-TAG-002/model"
    local ids
    _ids() { printf '%s' "$1" | grep -oE 'TC-MT-[A-Z]+-001' | sort | paste -sd, ; }

    SCENARIO_NAME="single --tag selects all elements carrying that tag"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    ids=$(_ids "$("$SYSCRIBE" -m "$M" list TestCase --tag integration 2>/dev/null)")
    [ "$ids" = "TC-MT-BOTH-001,TC-MT-INT-001" ] \
        && pass "--tag integration → BOTH + INT" || fail "got '$ids' (expected TC-MT-BOTH-001,TC-MT-INT-001)"

    SCENARIO_NAME="repeated --tag narrows with AND semantics"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    ids=$(_ids "$("$SYSCRIBE" -m "$M" list TestCase --tag integration --tag safety 2>/dev/null)")
    [ "$ids" = "TC-MT-BOTH-001" ] \
        && pass "--tag integration --tag safety → only the both-tagged TC" || fail "got '$ids' (expected TC-MT-BOTH-001)"

    SCENARIO_NAME="an element carrying only one of the tags is excluded"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    out=$("$SYSCRIBE" -m "$M" list TestCase --tag integration --tag safety 2>/dev/null)
    { ! printf '%s' "$out" | grep -qF "TC-MT-INT-001" && ! printf '%s' "$out" | grep -qF "TC-MT-SAFE-001"; } \
        && pass "single-tag TCs excluded under AND" || fail "a single-tag TC leaked into the AND result"

    SCENARIO_NAME="no --tag lists all (filter inactive)"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    ids=$(_ids "$("$SYSCRIBE" -m "$M" list TestCase 2>/dev/null)")
    [ "$ids" = "TC-MT-BOTH-001,TC-MT-INT-001,TC-MT-SAFE-001" ] \
        && pass "no --tag → all three TestCases" || fail "got '$ids' (expected all three)"
}
