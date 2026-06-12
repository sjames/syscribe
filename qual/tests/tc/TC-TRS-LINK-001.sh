tc_TRS_LINK_001() {
    local F="$1"
    local LINKED="$F/TC-TRS-LINK-001/linked"
    local NONE="$F/TC-TRS-LINK-001/none"
    local BASE="https://github.com/acme/uav/blob/main/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "base_url resolves file-backed elements (export carries url)"
    local out; out=$("$SYSCRIBE" -m "$LINKED" export 2>/dev/null)
    printf '%s' "$out" | grep -qF "\"url\": \"$BASE/UAV/Avionics/FlightController.md\"" \
        && pass "FlightController resolves to hosted URL under base_url" \
        || fail "FlightController hosted URL not found in export"
    printf '%s' "$out" | grep -qF "\"url\": \"$BASE/Requirements/SafeLanding.md\"" \
        && pass "Requirement resolves to hosted URL" \
        || fail "Requirement hosted URL not found in export"

    _scn "no [links] table yields no URLs"
    local none_out; none_out=$("$SYSCRIBE" -m "$NONE" export 2>/dev/null)
    printf '%s' "$none_out" | grep -q '"url"' \
        && fail "unconfigured model unexpectedly carries a url" \
        || pass "no url resolved without [links] config"
}
