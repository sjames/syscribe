tc_TRS_BT_025() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    _scn "an unresolved required parameter fails the export, naming it"
    rc=0; out=$(exp CONF-UAV-BUILD-001) || rc=$?
    [ "$rc" -eq 1 ] && pass "exits non-zero" || fail "exit $rc"
    printf '%s' "$out" | grep -q 'FEAT-UAV-PROP.motorKV' && pass "the parameter is named" || fail "not named: $out"

    _scn "--allow-open writes a marked placeholder instead"
    rc=0; "$SYSCRIBE" -m "$U" params export --config CONF-UAV-BUILD-001 --allow-open --out "$T/open.yaml" >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 0 ] && pass "exits zero" || fail "exit $rc"
    grep -E '^ +motorKV: null +# OPEN' "$T/open.yaml" >/dev/null && pass "an explicit OPEN placeholder" || fail "no placeholder: $(cat "$T/open.yaml")"
    rm -rf "$T"
}
