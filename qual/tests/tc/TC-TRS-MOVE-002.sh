tc_TRS_MOVE_002() {
    local F="$1"; local BASE="$F/move/base"
    local W

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _grep() { grep -qF "$2" "$1"; }

    # Move a single element; check all reference kinds are rewritten.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::Sub::Widget Pkg::Other::Widget >/dev/null 2>&1 || true

    _scn "supertype reference rewritten"
    _grep "$W/Pkg/Sub/WidgetExtended.md" "supertype: Pkg::Other::Widget" && pass "supertype updated" || fail "supertype not updated"

    _scn "feature typedBy reference rewritten"
    _grep "$W/Pkg/Consumer.md" "typedBy: Pkg::Other::Widget" && pass "typedBy updated" || fail "typedBy not updated"

    _scn "nested connection endpoint rewritten"
    _grep "$W/Pkg/Consumer.md" "to: Pkg::Other::Widget::port" && pass "endpoint updated" || fail "endpoint not updated"

    _scn "prefix-sharing sibling left untouched"
    _grep "$W/Pkg/Thing.md" "supertype: Pkg::Sub::WidgetExtended" && pass "WidgetExtended untouched" || fail "WidgetExtended wrongly rewritten"

    _scn "body qualified-name reference rewritten"
    _grep "$W/Decisions.md" "Pkg::Other::Widget" && pass "ADR body reference updated" || fail "ADR body reference not updated"

    _scn "body prefix-sharing sibling left untouched"
    _grep "$W/Decisions.md" "Pkg::Sub::WidgetExtended" && pass "ADR body sibling untouched" || fail "ADR body sibling wrongly rewritten"

    _scn "companion SVG references rewritten"
    { _grep "$W/Diagram.svg" 'sysml:ref="Pkg::Other::Widget"' \
        && _grep "$W/Diagram.svg" 'sysml:ref="Pkg::Other::Widget::port"' \
        && _grep "$W/Diagram.svg" 'data-qname="Pkg::Other::Widget"'; } \
        && pass "SVG refs (ref/endpoint/data-qname) updated" || fail "SVG refs not fully updated"

    _scn "companion SVG sibling left untouched"
    _grep "$W/Diagram.svg" 'sysml:ref="Pkg::Sub::WidgetExtended"' && pass "SVG sibling untouched" || fail "SVG sibling wrongly rewritten"

    _scn "no unresolved references after move"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$W" validate 2>/dev/null) && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_no_code "E102"
    rm -rf "$W"

    # Package move: descendant endpoint follows the move.
    W=$(mktemp -d); cp -r "$BASE"/. "$W"/
    "$SYSCRIBE" -m "$W" move Pkg::Sub Pkg::Moved >/dev/null 2>&1 || true
    _scn "descendant endpoint follows package move"
    _grep "$W/Pkg/Consumer.md" "to: Pkg::Moved::Widget::port" && pass "descendant endpoint updated" || fail "descendant endpoint not updated"
    rm -rf "$W"
}
