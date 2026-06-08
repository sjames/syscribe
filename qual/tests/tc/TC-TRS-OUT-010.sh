tc_TRS_OUT_010() {
    local F="$1"
    local M="$F/TC-TRS-OUT-010/model"
    local PARENT="ConnDemo::Powertrain"
    local ROOT="ConnDemoModel"   # model-root package element

    # Scenario 1: text output rooted at the parent names both sub-parts.
    SCENARIO_NAME="text tree names both wired sub-parts"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    SCENARIO_OUTPUT=$("$SYSCRIBE" -m "$M" connectivity "$PARENT" 2>/dev/null)
    SCENARIO_EXIT=$?
    assert_exit_zero
    printf '%s' "$SCENARIO_OUTPUT" | grep -qF "$PARENT" \
        && pass "text rooted at $PARENT" || fail "text not rooted at $PARENT"
    assert_output_contains "ConnDemo::Battery"
    assert_output_contains "ConnDemo::Motor"

    # Scenario 2: --format json is valid JSON with nodes + a connection edge
    # between the two sub-parts.
    SCENARIO_NAME="json exposes nodes and a connection edge between sub-parts"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local js; js=$("$SYSCRIBE" -m "$M" connectivity "$PARENT" --format json 2>/dev/null)
    printf '%s' "$js" | jq -e . >/dev/null 2>&1 \
        && pass "json is valid" || fail "json is not valid"
    local nn; nn=$(printf '%s' "$js" | jq -r '.nodes | length')
    [ "$nn" -ge 3 ] && pass "nodes has $nn entries" || fail "nodes length=$nn (expected >=3)"
    local ce; ce=$(printf '%s' "$js" | jq -r \
        '[.edges[] | select(.kind=="connection" and .from=="ConnDemo::Battery" and .to=="ConnDemo::Motor")] | length')
    [ "$ce" = "1" ] && pass "connection edge Battery->Motor present" \
        || fail "connection edge Battery->Motor count=$ce (expected 1)"

    # Scenario 3: --format dot is styled Graphviz.
    SCENARIO_NAME="dot output is styled Graphviz"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local dot; dot=$("$SYSCRIBE" -m "$M" connectivity "$PARENT" --format dot 2>/dev/null)
    printf '%s' "$dot" | grep -qF "digraph" \
        && pass "dot contains digraph" || fail "dot missing digraph"
    printf '%s' "$dot" | grep -qE "shape=" \
        && pass "dot carries shape attribute" || fail "dot missing shape attribute"
    printf '%s' "$dot" | grep -qE "peripheries=" \
        && pass "dot carries peripheries attribute" || fail "dot missing peripheries attribute"

    # Scenario 4: the model-root element dumps the whole model.
    SCENARIO_NAME="model-root element dumps the whole model"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local total; total=$("$SYSCRIBE" -m "$M" export 2>/dev/null | jq -r '.elements | length')
    local reached; reached=$("$SYSCRIBE" -m "$M" connectivity "$ROOT" --format json 2>/dev/null | jq -r '.nodes | length')
    [ "$reached" = "$total" ] && pass "reached $reached of $total elements" \
        || fail "reached $reached of $total elements (expected whole model)"

    # Scenario 5: --depth bounds the walk.
    SCENARIO_NAME="depth bounds the walk"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local d0; d0=$("$SYSCRIBE" -m "$M" connectivity "$PARENT" --depth 0 --format json 2>/dev/null | jq -r '.nodes | length')
    [ "$d0" = "1" ] && pass "--depth 0 yields only the root node" \
        || fail "--depth 0 nodes=$d0 (expected 1)"
    local d0_text; d0_text=$("$SYSCRIBE" -m "$M" connectivity "$PARENT" --depth 0 2>/dev/null)
    printf '%s' "$d0_text" | grep -qF "ConnDemo::Battery" \
        && fail "--depth 0 unexpectedly shows sub-parts" || pass "--depth 0 hides sub-parts"

    # Scenario 6: an unknown root exits non-zero.
    SCENARIO_NAME="unknown root exits non-zero"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    "$SYSCRIBE" -m "$M" connectivity "Nope::DoesNotExist" >/dev/null 2>&1 && SCENARIO_EXIT=0 || SCENARIO_EXIT=$?
    assert_exit_nonzero
}
