tc_TRS_LINK_003() {
    local F="$1"
    local LINKED="$F/TC-TRS-LINK-001/linked"
    local NONE="$F/TC-TRS-LINK-001/none"
    local URL="https://github.com/acme/uav/blob/main/model/Requirements/SafeLanding.md"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    _scn "configured links append a Mermaid click directive per node"
    local mmd; mmd=$("$SYSCRIBE" -m "$LINKED" render Diagrams/Trace.md 2>/dev/null)
    printf '%s' "$mmd" | grep -qF "click SAFE \"$URL\" _blank" \
        && pass "click directive emitted with hosted URL" \
        || fail "no click <node> \"<hosted url>\" _blank directive found"

    _scn "no [links] table emits no hosted click directive"
    local mmd2; mmd2=$("$SYSCRIBE" -m "$NONE" render Diagrams/Trace.md 2>/dev/null)
    printf '%s' "$mmd2" | grep -q 'click .*"https' \
        && fail "unconfigured Mermaid unexpectedly carries a hosted click" \
        || pass "no hosted click directive without [links] config"
}
