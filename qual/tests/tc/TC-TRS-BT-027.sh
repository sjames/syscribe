tc_TRS_BT_027() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    local B="$F/TC-TRS-BT-027"
    _scn "an unresolvable consumedBy is reported like an unresolvable implementedBy"
    out=$("$SYSCRIBE" -m "$B/consumers" validate 2>&1 || true)
    [ "$(printf '%s' "$out" | grep W023 | grep -c 'does_not_exist_anywhere')" -eq 1 ] && pass "W023 for the missing local path" || fail "expected W023: $out"
    [ "$(printf '%s' "$out" | grep -c W023)" -eq 1 ] && pass "an existing path and a remote URI are accepted" || fail "extra W023: $out"

    _scn "trace lists the consumer for the parameter"
    out=$("$SYSCRIBE" -m "$B/consumers" trace FEAT-UAV-TELEM 2>&1 || true)
    printf '%s' "$out" | grep -E '\| rateHz \|.*Cargo.toml' >/dev/null && pass "trace shows who reads rateHz" || fail "no consumer in trace: $out"

    _scn "params export shows the link"
    out=$("$SYSCRIBE" -m "$B/consumers" params export --config CONF-UAV-FLIGHT-001 2>&1)
    printf '%s' "$out" | grep -q 'rateHz is read by: repo:Cargo.toml' && pass "export names the consumer" || fail "export has no consumer: $out"
    rm -rf "$T"
}
