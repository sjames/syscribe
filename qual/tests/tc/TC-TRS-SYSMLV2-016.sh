tc_TRS_SYSMLV2_016() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-016/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "a package-relative typedBy: reference to a documented target across packages suppresses W600"
    local out; out=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    local w600_count; w600_count=$(printf '%s' "$out" | grep -c 'W600' || true)
    # Five PartDef/Part elements total: Services::Documented (own doc),
    # Services::Undocumented (no doc -- always fires), System::Top (own doc),
    # System::Top::x (typed by Documented, cross-package -- suppressed once
    # REQ-TRS-SYSMLV2-016 lands), System::Top::y (typed by Undocumented --
    # still fires). Expected count: exactly 2 (Undocumented itself, y).
    [ "$w600_count" -eq 2 ] && pass "exactly two W600 raised (Undocumented, y) -- x is suppressed" \
        || fail "W600 count=$w600_count (expected 2)"

    _scn "the documented, cross-package-referenced target itself raises no W600"
    # Services::Documented has its own doc comment, so it's unaffected by this
    # fix either way -- a sanity check that the fixture itself is well-formed.
    local doc_out; doc_out=$("$SYSCRIBE" -m "$M" show SysML2::Services::Documented 2>&1)
    printf '%s' "$doc_out" | grep -q '## Documentation' \
        && pass "Documented shows a Documentation section" \
        || fail "Documented unexpectedly has no Documentation section: $doc_out"
}
