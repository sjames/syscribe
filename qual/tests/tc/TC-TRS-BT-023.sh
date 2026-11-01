tc_TRS_BT_023() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    _scn "a load-stage export carries exactly the items whose window contains load"
    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-DEPLOY-001 --out "$T/load.yaml" >/dev/null 2>&1
    [ "$(yq "$T/load.yaml" "sorted(d['features'])")" = "['FEAT-UAV-TELEM']" ] && pass "toggles: the load..runtime Telemetry" || fail "toggles wrong: $(cat "$T/load.yaml")"
    [ "$(yq "$T/load.yaml" "{k:sorted(v) for k,v in sorted(d['parameters'].items())}")" = "{'FEAT-UAV-PROP': ['maxCurrentA', 'motorKV'], 'FEAT-UAV-TELEM': ['mode']}" ] \
        && pass "parameters: load-window ones only (rateHz is runtime-only)" || fail "parameters wrong: $(cat "$T/load.yaml")"

    _scn "a compile-only item is absent once the stage has passed"
    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-FLIGHT-001 --out "$T/run.yaml" >/dev/null 2>&1
    [ "$(yq "$T/run.yaml" "'motorKV' in d['parameters']['FEAT-UAV-PROP']")" = "False" ] && pass "motorKV (compile..load) is not in a runtime export" || fail "motorKV leaked: $(cat "$T/run.yaml")"
    [ "$(yq "$T/run.yaml" "d['parameters']['FEAT-UAV-TELEM']['rateHz']")" = "20" ] && pass "the Configuration's own runtime binding" || fail "rateHz wrong"

    _scn "entries are keyed by FEAT id, not qualified name"
    grep -v '^#' "$T/load.yaml" | sed 's/#.*//' | grep -q '::' && fail "a qualified name is a key" || pass "keys are FEAT-* ids"

    _scn "the value resolves in order: binding, then default, then fallback"
    [ "$(yq "$T/load.yaml" "d['parameters']['FEAT-UAV-PROP']['motorKV']")" = "1000" ] && pass "inherited binding" || fail "motorKV wrong"
    [ "$(yq "$T/load.yaml" "d['parameters']['FEAT-UAV-TELEM']['mode']")" = "sport" ] && pass "own binding" || fail "mode wrong"
    rm -rf "$T"
}
