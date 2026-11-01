tc_TRS_BT_029() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    chk() { "$SYSCRIBE" -m "$U" params check "$@" --config CONF-UAV-DEPLOY-001 2>&1 || true; }
    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-DEPLOY-001 --out "$T/ok.yaml" >/dev/null 2>&1

    _scn "a clean generated file passes"
    rc=0; out=$("$SYSCRIBE" -m "$U" params check "$T/ok.yaml" --config CONF-UAV-DEPLOY-001 2>&1) || rc=$?
    [ "$rc" -eq 0 ] && pass "exit zero" || fail "exit $rc: $out"

    _scn "an out-of-range value fails, naming the key"
    sed 's/^\( *motorKV:\) 1000 /\1 5000 /' "$T/ok.yaml" >"$T/bad.yaml"
    rc=0; out=$("$SYSCRIBE" -m "$U" params check "$T/bad.yaml" --config CONF-UAV-DEPLOY-001 2>&1) || rc=$?
    [ "$rc" -eq 1 ] && pass "exit non-zero" || fail "exit $rc"
    printf '%s' "$out" | grep E205 | grep -q 'parameters.FEAT-UAV-PROP.motorKV' && pass "E205 names the key" || fail "E205 missing: $out"

    _scn "an enum value outside enumValues, a wrong type, an unknown key"
    sed 's/^\( *mode:\) sport /\1 turbo /' "$T/ok.yaml" >"$T/e.yaml"
    out=$(chk "$T/e.yaml"); printf '%s' "$out" | grep E206 | grep -q 'mode' && pass "E206" || fail "no E206: $out"
    sed 's/^\( *maxCurrentA:\) 45 /\1 lots /' "$T/ok.yaml" >"$T/t.yaml"
    out=$(chk "$T/t.yaml"); printf '%s' "$out" | grep E761 | grep -q 'maxCurrentA' && pass "E761 for a wrong type" || fail "no E761: $out"
    { cat "$T/ok.yaml"; printf 'surprise: 1\n'; } >"$T/u.yaml"
    out=$(chk "$T/u.yaml"); printf '%s' "$out" | grep E760 | grep -q 'surprise' && pass "E760 for an unknown top-level key" || fail "no E760: $out"
    sed 's/^    mode: sport/    modez: sport/' "$T/ok.yaml" >"$T/u2.yaml"
    out=$(chk "$T/u2.yaml"); printf '%s' "$out" | grep E760 | grep -q 'modez' && pass "E760 for an unknown parameter" || fail "no E760 (param): $out"

    _scn "a missing required parameter"
    grep -v '^ *motorKV:' "$T/ok.yaml" >"$T/m.yaml"
    out=$(chk "$T/m.yaml"); printf '%s' "$out" | grep E762 | grep -q 'motorKV' && pass "E762 for a missing required parameter" || fail "no E762: $out"

    _scn "a fixed parameter given a different value"
    cp -r "$U" "$T/fx"; sed -i 's/^    unit: A$/    unit: A\n    value: 40/' "$T/fx/Features/Propulsion.md"
    sed -i '/maxCurrentA: 45/d' "$T/fx/Configurations/CONF-UAV-DEPLOY-001.md"
    "$SYSCRIBE" -m "$T/fx" params export --config CONF-UAV-DEPLOY-001 --out "$T/fx.yaml" >/dev/null 2>&1
    sed 's/^\( *maxCurrentA:\) 40 /\1 41 /' "$T/fx.yaml" >"$T/fx2.yaml"
    out=$("$SYSCRIBE" -m "$T/fx" params check "$T/fx2.yaml" --config CONF-UAV-DEPLOY-001 2>&1 || true)
    printf '%s' "$out" | grep -q 'E204' && pass "E204 for a fixed parameter" || fail "no E204: $out"
    rm -rf "$T"
}
