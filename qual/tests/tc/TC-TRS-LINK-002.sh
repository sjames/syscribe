tc_TRS_LINK_002() {
    local F="$1"
    local LINKED="$F/TC-TRS-LINK-001/linked"
    local NONE="$F/TC-TRS-LINK-001/none"
    local BASE="https://github.com/acme/uav/blob/main/model"
    local FC_URL="$BASE/UAV/Avionics/FlightController.md"
    local REQ_URL="$BASE/Requirements/SafeLanding.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "configured links wrap each SVG shape in an SVG hyperlink"
    local svg; svg=$("$SYSCRIBE" -m "$LINKED" diagram export Diagrams::Pinned --format svg 2>/dev/null)
    grep -qF 'xmlns:xlink="http://www.w3.org/1999/xlink"' <<<"$svg" \
        && pass "SVG declares the xlink namespace" \
        || fail "SVG lacks xmlns:xlink"
    grep -qF "<a xlink:href=\"$FC_URL\" href=\"$FC_URL\" target=\"_blank\" rel=\"noopener\">" <<<"$svg" \
        && pass "FlightController shape wrapped in <a xlink:href href target=_blank rel=noopener>" \
        || fail "FlightController wrapper missing: $svg"
    grep -qF "<a xlink:href=\"$REQ_URL\" href=\"$REQ_URL\" target=\"_blank\" rel=\"noopener\">" <<<"$svg" \
        && pass "SafeLanding requirement shape wrapped in its hosted-URL anchor" \
        || fail "SafeLanding wrapper missing"
    # The anchor wraps the shape group: the <a> is immediately followed by the <g sysml:ref>.
    grep -qzF "<a xlink:href=\"$FC_URL\" href=\"$FC_URL\" target=\"_blank\" rel=\"noopener\">
  <g id=\"s-fc\"" <<<"$svg" \
        && pass "the anchor encloses the shape group <g id=s-fc>" \
        || fail "anchor does not directly enclose the shape group"
    local anchors; anchors=$(grep -c '<a ' <<<"$svg")
    [ "$anchors" -eq 2 ] && pass "exactly one anchor per linked shape ($anchors)" \
        || fail "expected 2 anchors, found $anchors"

    _scn "configured links add Mermaid click directives"
    local mmd; mmd=$("$SYSCRIBE" -m "$LINKED" diagram export Diagrams::Pinned --format mermaid 2>/dev/null)
    grep -qF "click s_fc href \"$FC_URL\" _blank" <<<"$mmd" \
        && pass "click line for the FlightController node" \
        || fail "no click line for s_fc: $mmd"
    grep -qF "%% ref: UAV::Avionics::FlightController" <<<"$mmd" \
        && pass "generated Mermaid carries %% ref: annotations" \
        || fail "no %% ref: annotation"

    _scn "no [links] table leaves shapes unwrapped and Mermaid without click lines"
    local nsvg; nsvg=$("$SYSCRIBE" -m "$NONE" diagram export Diagrams::Pinned --format svg 2>/dev/null)
    grep -qF 'sysml:ref="UAV::Avionics::FlightController"' <<<"$nsvg" \
        && pass "SVG still drawn without [links]" \
        || fail "SVG not drawn without [links]"
    grep -q '<a ' <<<"$nsvg" \
        && fail "unconfigured model unexpectedly wraps shapes in <a>" \
        || pass "no <a> wrappers without [links]"
    local nmmd; nmmd=$("$SYSCRIBE" -m "$NONE" diagram export Diagrams::Pinned --format mermaid 2>/dev/null)
    grep -q 'click ' <<<"$nmmd" \
        && fail "unconfigured model unexpectedly emits click lines" \
        || pass "no click lines without [links]"
}
