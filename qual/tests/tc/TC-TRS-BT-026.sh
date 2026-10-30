tc_TRS_BT_026() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    local B="$F/TC-TRS-BT-026"
    _scn "a fallback outside range or enumValues is an error"
    out=$("$SYSCRIBE" -m "$B/bad" validate 2>&1 || true)
    [ "$(printf '%s' "$out" | grep E759 | grep -c 'rateHz')" -eq 1 ] && pass "E759 for a fallback outside range" || fail "expected E759 for rateHz: $out"
    [ "$(printf '%s' "$out" | grep E759 | grep -c 'mode')" -eq 1 ] && pass "E759 for a fallback outside enumValues" || fail "expected E759 for mode: $out"
    out=$("$SYSCRIBE" -m "$B/bad" feature-check 2>&1 || true)
    [ "$(printf '%s' "$out" | grep -c E759)" -eq 2 ] && pass "feature-check reports each once" || fail "feature-check E759 count: $out"

    _scn "a valid fallback appears in the export"
    exp CONF-UAV-FLIGHT-001 >"$T/f.yaml"
    grep -E 'rateHz: 20 .*fallback: 5' "$T/f.yaml" >/dev/null && pass "fallback shown as a comment" || fail "fallback missing: $(cat "$T/f.yaml")"
    out=$("$SYSCRIBE" -m "$U" validate 2>&1 || true); printf '%s' "$out" | grep -q E759 && fail "E759 on a valid fallback" || pass "a valid fallback is clean"

    _scn "a fallback stands in for a missing value"
    cp -r "$U" "$T/nofb"; sed -i '/^    default: 10$/d; /^    fallback: 5$/{s/.*/    fallback: 5/}' "$T/nofb/Features/Telemetry.md"
    sed -i '/Telemetry.rateHz: 20/d' "$T/nofb/Configurations/CONF-UAV-FLIGHT-001.md"
    out=$("$SYSCRIBE" -m "$T/nofb" params export --config CONF-UAV-FLIGHT-001 2>&1)
    printf '%s' "$out" | grep -E 'rateHz: 5 .*source: fallback' >/dev/null && pass "the fallback is the value, labelled" || fail "fallback not used: $out"
    rm -rf "$T"
}
