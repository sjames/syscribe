tc_TRS_FTA_002() {
    local F="$1"; local R="$F/TC-TRS-FTA-002/resolved"; local D="$F/TC-TRS-FTA-002/dangling"
    local o

    run_scenario "resolving ref (qname and id) is accepted" "$R"
    assert_no_code "W047"
    assert_no_code "E927"

    run_scenario "dangling ref raises E927" "$D"
    assert_has_code "E927"
    assert_output_contains "Arch::NoSuchValve"
    assert_no_code "W047"

    run_scenario "show, links and fault-tree render surface the ref" "$R"
    o=$("$SYSCRIBE" -m "$R" show FTE-FREF-001 2>/dev/null || true)
    grep -qF "| **ref** | Arch::SolenoidValve |" <<<"$o" \
        && pass "show lists ref" || fail "show does not list ref"
    o=$("$SYSCRIBE" -m "$R" links Arch::SolenoidValve 2>/dev/null || true)
    grep -qF "| FT-FREF-001::FTE-FREF-001 | ref |" <<<"$o" \
        && pass "links lists the event as an inbound ref source" || fail "links missing inbound ref from FTE-FREF-001"
    o=$("$SYSCRIBE" -m "$R" links FTE-FREF-002 2>/dev/null || true)
    grep -qF "| ref | REQ-FREF-001 | Requirement |" <<<"$o" \
        && pass "links lists the id-authored ref as a resolved outbound" || fail "links outbound ref by id not resolved"
    o=$("$SYSCRIBE" -m "$R" fault-tree render FT-FREF-001 2>/dev/null || true)
    grep -qF "Arch::SolenoidValve" <<<"$o" \
        && pass "fault-tree render names the referenced element" || fail "fault-tree render omits the ref"
}
