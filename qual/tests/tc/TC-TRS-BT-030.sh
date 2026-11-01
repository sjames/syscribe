tc_TRS_BT_030() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-DEPLOY-001 --out "$T/old.yaml" >/dev/null 2>&1

    _scn "a current file is not stale"
    out=$("$SYSCRIBE" -m "$U" params check "$T/old.yaml" --config CONF-UAV-DEPLOY-001 2>&1 || true)
    printf '%s' "$out" | grep -q W751 && fail "W751 on a current file" || pass "no staleness warning"

    _scn "changing a parameter's range in the model makes an older file stale"
    cp -r "$U" "$T/m2"; sed -i 's/range: "0..60"/range: "0..55"/' "$T/m2/Features/Propulsion.md"
    rc=0; out=$("$SYSCRIBE" -m "$T/m2" params check "$T/old.yaml" --config CONF-UAV-DEPLOY-001 2>&1) || rc=$?
    printf '%s' "$out" | grep -q W751 && pass "W751 reported" || fail "no W751: $out"
    [ "$rc" -eq 0 ] && pass "a warning alone does not fail" || fail "exit $rc"

    _scn "--deny W751 makes it a gate"
    rc=0; "$SYSCRIBE" -m "$T/m2" params check "$T/old.yaml" --config CONF-UAV-DEPLOY-001 --deny W751 >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 2 ] && pass "exit 2 under --deny W751" || fail "exit $rc"

    _scn "a changed binding is stale too"
    cp -r "$U" "$T/m3"; sed -i 's/maxCurrentA: 45/maxCurrentA: 46/' "$T/m3/Configurations/CONF-UAV-DEPLOY-001.md"
    out=$("$SYSCRIBE" -m "$T/m3" params check "$T/old.yaml" --config CONF-UAV-DEPLOY-001 2>&1 || true)
    printf '%s' "$out" | grep -q W751 && pass "W751 for a changed value" || fail "no W751: $out"
    sed 's/^# projection-hash:.*//' "$T/old.yaml" >"$T/nohash.yaml"
    out=$("$SYSCRIBE" -m "$U" params check "$T/nohash.yaml" --config CONF-UAV-DEPLOY-001 2>&1 || true)
    printf '%s' "$out" | grep -q W751 && pass "W751 when the hash header is missing" || fail "no W751 without a hash: $out"
    rm -rf "$T"
}
