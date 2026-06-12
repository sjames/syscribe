tc_TRS_LINK_004() {
    local F="$1"
    local LINKED="$F/TC-TRS-LINK-001/linked"
    local NONE="$F/TC-TRS-LINK-001/none"
    local URL="https://github.com/acme/uav/blob/main/model/UAV/Avionics/FlightController.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "configured links render report element references as Markdown links"
    local rep; rep=$("$SYSCRIBE" -m "$LINKED" 2>/dev/null)
    printf '%s' "$rep" | grep -qF "[UAV::Avionics::FlightController]($URL)" \
        && pass "satisfying element rendered as [<qname>](<hosted url>)" \
        || fail "no [<qname>](<hosted url>) Markdown link in report"

    _scn "export carries the hosted url"
    local exp; exp=$("$SYSCRIBE" -m "$LINKED" export 2>/dev/null)
    printf '%s' "$exp" | grep -qF "\"url\": \"$URL\"" \
        && pass "export carries hosted url for FlightController" \
        || fail "export missing hosted url for FlightController"

    _scn "no [links] table leaves the report unlinked"
    local rep2; rep2=$("$SYSCRIBE" -m "$NONE" 2>/dev/null)
    printf '%s' "$rep2" | grep -qF '](http' \
        && fail "unconfigured report unexpectedly carries a hosted link" \
        || pass "no hosted Markdown link without [links] config"
}
