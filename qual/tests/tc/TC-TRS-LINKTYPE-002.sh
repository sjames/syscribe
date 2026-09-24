tc_TRS_LINKTYPE_002() {
    local F="$1"; local FX="$F/TC-TRS-LINKTYPE-002"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    # has <output> <file-substring> <code>  — a finding line mentioning both
    has() { printf '%s' "$1" | grep -F -- "$2" | grep -qE -- "$3"; }

    local out
    out=$("$SYSCRIBE" -m "$FX/model" validate 2>&1 || true)
    _scn "a valid links: entry validates cleanly"
    has "$out" "Arch/Good.md" "E63[0-9]|W047" && fail "unexpected finding on Good.md" || pass "Good.md clean"

    _scn "an undeclared link type raises E630 naming the declared types"
    has "$out" "Undeclared.md" "E630" && pass "E630 raised" || fail "E630 not raised"
    printf '%s' "$out" | grep 'Undeclared.md' | grep 'E630' | grep -q 'informs' && pass "E630 lists declared types" || fail "E630 does not list declared types"

    _scn "a malformed links: shape raises E631"
    has "$out" "NotAMap.md" "E631" && pass "E631 for list-shaped links:" || fail "no E631 for list-shaped links:"
    has "$out" "BadValue.md" "E631" && pass "E631 for mapping value" || fail "no E631 for mapping value"

    _scn "a dangling target raises E632"
    has "$out" "Dangling.md" "E632" && pass "E632 raised" || fail "E632 not raised"

    _scn "links: with no [linkTypes] table raises E630 with a declaration hint"
    out=$("$SYSCRIBE" -m "$FX/unconfigured" validate 2>&1 || true)
    has "$out" "UsesLinks.md" "E630" && pass "E630 raised when unconfigured" || fail "no E630 when unconfigured"
    printf '%s' "$out" | grep 'E630' | grep -qi 'linkTypes' && pass "hint mentions [linkTypes]" || fail "no [linkTypes] hint"

    _scn "in a configuration lens a links: target inactive in the variant escapes as W019, not E632"
    out=$("$SYSCRIBE" -m "$FX/variant" validate --config CONF-M0-BASE-001 2>&1) && pass "lens exit zero" || fail "lens non-zero exit: $out"
    printf '%s' "$out" | grep 'ReqCore.md' | grep 'W019' | grep -q 'REQ-V5-WDT-002' && pass "W019 for inactive links: target" || fail "no W019: $out"
    printf '%s' "$out" | grep -q 'E632' && fail "E632 raised in lens: $out" || pass "no E632 in lens"
}
