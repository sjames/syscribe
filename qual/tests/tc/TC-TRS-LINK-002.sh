tc_TRS_LINK_002() {
    local F="$1"
    local LINKED="$F/TC-TRS-LINK-001/linked"
    local NONE="$F/TC-TRS-LINK-001/none"
    local URL="https://github.com/acme/uav/blob/main/model/Requirements/SafeLanding.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "configured links wrap SVG shapes in a hosted hyperlink"
    local svg; svg=$("$SYSCRIBE" -m "$LINKED" diagram req REQ-UAV-SAFE-001 --show-satisfy 2>/dev/null)
    printf '%s' "$svg" | grep -qF "<a xlink:href=\"$URL\" href=\"$URL\" target=\"_blank\"" \
        && pass "shape wrapped in <a xlink:href/href target=_blank> to hosted URL" \
        || fail "no hosted <a> wrapper with the requirement URL in SVG"
    printf '%s' "$svg" | grep -qF 'xmlns:xlink=' \
        && pass "SVG declares the xlink namespace" \
        || fail "SVG missing xmlns:xlink declaration"

    _scn "no [links] table leaves shapes unwrapped by hosted links"
    local svg2; svg2=$("$SYSCRIBE" -m "$NONE" diagram req REQ-UAV-SAFE-001 --show-satisfy 2>/dev/null)
    printf '%s' "$svg2" | grep -q 'xlink:href="https' \
        && fail "unconfigured SVG unexpectedly carries a hosted hyperlink" \
        || pass "no hosted <a> wrapper without [links] config"
}
