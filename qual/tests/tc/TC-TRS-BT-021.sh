tc_TRS_BT_021() {
    local F="$1"; local B="$F/TC-TRS-BT-021"
    local out
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    pj() { "$SYSCRIBE" -m "$B/chain" configure CONF-BT-B1 --json 2>/dev/null | jq -e "$1" >/dev/null; }

    _scn "a value inherited from a base reports that base and its stage"
    pj '.provenance.parameters[]|select(.key=="Features::Motor.kv")|.source.kind=="inherited" and .source.from=="CONF-BT-A1" and .stage=="compile" and .value==1000' \
        && pass "kv: inherited from CONF-BT-A1 at compile" || fail "kv provenance wrong"
    pj '.provenance.parameters[]|select(.key=="Features::Motor.spin")|.source.kind=="own" and .stage=="load" and .value==50' \
        && pass "spin: own at load" || fail "spin provenance wrong"

    _scn "model-level sources carry no stage"
    pj '.provenance.parameters[]|select(.key=="Features::Motor.lim")|.source.kind=="fixed" and .stage==null' && pass "fixed value" || fail "fixed wrong"
    pj '.provenance.parameters[]|select(.key=="Features::Motor.dflt")|.source.kind=="default" and .value==3' && pass "default value" || fail "default wrong"
    pj '.provenance.parameters[]|select(.key=="Features::Motor.ratio")|.source.kind=="derived" and .source.expr=="kv * 2"' && pass "derived expression" || fail "derived wrong"

    _scn "a propagated value reports the bindTo source"
    pj '.provenance.parameters[]|select(.key=="Features::Comp.kv")|.source.kind=="bindTo" and .source.from=="Features::Motor.kv" and .value==1000' \
        && pass "Comp.kv: bindTo Features::Motor.kv" || fail "bindTo provenance wrong"

    _scn "selections report their source and stage"
    pj '.provenance.selections[]|select(.feature=="Features::Motor")|.source.kind=="inherited" and .source.from=="CONF-BT-A1" and .stage=="compile"' \
        && pass "Motor: inherited at compile" || fail "Motor selection provenance wrong"
    pj '.provenance.selections[]|select(.feature=="Features::Sensor")|.source.kind=="own" and .stage=="load"' \
        && pass "Sensor: own at load" || fail "Sensor selection provenance wrong"
    pj '.provenance.bindingStage=="load"' && pass "the Configuration's own stage" || fail "stage wrong"
}
