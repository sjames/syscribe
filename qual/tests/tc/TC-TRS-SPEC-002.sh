tc_TRS_SPEC_002() {
    local OUT; OUT=$("$SYSCRIBE" spec types 2>/dev/null)

    SCENARIO_NAME="spec types explains the interface/connection relationship"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    if printf '%s' "$OUT" | grep -qiF "connection whose ends are ports" \
       && printf '%s' "$OUT" | grep -qiE "InterfaceDef.*kind of.*ConnectionDef"; then
        pass "interface-is-a-connection-of-ports stated"
    else
        fail "interface/connection relationship not stated in spec types"
    fi

    SCENARIO_NAME="spec types explains conjugation"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    printf '%s' "$OUT" | grep -qiE "conjugat" && printf '%s' "$OUT" | grep -qiE "flip|in.*out|reverse" \
        && pass "conjugation direction-flip stated" \
        || fail "conjugation rule not stated in spec types"

    SCENARIO_NAME="spec types distinguishes the constructs"; printf "  ▶ %s\n" "$SCENARIO_NAME"
    local ok=1
    for c in PortDef Port InterfaceDef ConnectionDef; do
        printf '%s' "$OUT" | grep -qF "$c" || ok=0
    done
    [ "$ok" -eq 1 ] && pass "PortDef/Port/InterfaceDef/ConnectionDef all named" || fail "a construct is not named in the guide"
}
