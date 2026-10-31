tc_TRS_BT_000() {
    local F="$1"
    local R; R="$(cd "$F/../.." && pwd)"
    local EX="$R/examples/staged-configuration/model"
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local W; W="$(mktemp -d)"
    local out rc

    _scn "a compile, a load and a runtime Configuration chain validates and reports residual variability"
    out=$("$SYSCRIBE" -m "$EX" validate 2>&1 || true)
    printf '%s' "$out" | grep -q '0 errors, 0 warnings' && pass "the three-stage model validates clean" || fail "not clean: $out"
    out=$("$SYSCRIBE" -m "$EX" residual --config CONF-UAV-BUILD-001 --json 2>&1 || true)
    printf '%s' "$out" | jq -e '[.groups[].items[]] | length > 0' >/dev/null 2>&1 && pass "the compile Configuration leaves later-stage items open" || fail "no residual on compile: $out"
    out=$("$SYSCRIBE" -m "$EX" residual --config CONF-UAV-FLIGHT-001 --json 2>&1 || true)
    printf '%s' "$out" | jq -e '[.groups[] | select(.closed|not) | .items[]] | length == 0' >/dev/null 2>&1 && pass "the runtime Configuration leaves nothing open" || fail "runtime residual not empty: $out"

    _scn "the runtime Configuration exports a parameter file that checks clean"
    rc=0; "$SYSCRIBE" -m "$EX" params export --config CONF-UAV-FLIGHT-001 --schema "$W/s.json" > "$W/flight.yaml" 2>"$W/err" || rc=$?
    [ "$rc" -eq 0 ] && pass "params export succeeds" || fail "export failed ($rc): $(cat "$W/err")"
    python3 -I - "$W/flight.yaml" "$W/s.json" <<'PY' && pass "the file validates against the generated schema" || fail "schema validation failed"
import sys, json, yaml, jsonschema
jsonschema.validate(yaml.safe_load(open(sys.argv[1])), json.load(open(sys.argv[2])))
PY
    rc=0; "$SYSCRIBE" -m "$EX" params check "$W/flight.yaml" --config CONF-UAV-FLIGHT-001 >/dev/null 2>&1 || rc=$?
    [ "$rc" -eq 0 ] && pass "params check exits zero" || fail "params check exited $rc"

    _scn "a late edit to the model makes the exported file stale"
    cp -r "$EX" "$W/model"
    sed -i 's/range: "1..50"/range: "1..60"/' "$W/model/Features/Telemetry.md"
    out=$("$SYSCRIBE" -m "$W/model" params check "$W/flight.yaml" --config CONF-UAV-FLIGHT-001 2>&1 || true)
    printf '%s' "$out" | grep -q 'W751' && pass "W751 reported for the old file" || fail "no staleness warning: $out"
    rm -rf "$W"
}
