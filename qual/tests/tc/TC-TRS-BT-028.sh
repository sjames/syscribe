tc_TRS_BT_028() {
    local F="$1"
    local out rc
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    local U="$F/TC-TRS-BT-023/uav"
    local T; T=$(mktemp -d)
    exp() { "$SYSCRIBE" -m "$U" params export --config "$@" 2>&1; }
    # yq <file> <python expr over d> — evaluate against the parsed YAML
    yq() { python3 -I -c 'import sys,yaml;d=yaml.safe_load(open(sys.argv[1]));print(eval(sys.argv[2]))' "$1" "$2"; }

    local schema="$T/schema.json"
    js() { python3 -I -c 'import sys,json,yaml,jsonschema;jsonschema.validate(yaml.safe_load(open(sys.argv[1])),json.load(open(sys.argv[2])))' "$@" 2>&1; }
    python3 -I -c 'import jsonschema,yaml' 2>/dev/null || { fail "python3 with the jsonschema and yaml modules is required"; rm -rf "$T"; return; }

    _scn "a generated file validates against its schema (a standard validator)"
    for conf in CONF-UAV-DEPLOY-001 CONF-UAV-FLIGHT-001; do
        "$SYSCRIBE" -m "$U" params export --config "$conf" --out "$T/v.yaml" --schema "$schema" >/dev/null 2>&1
        out=$(js "$T/v.yaml" "$schema") && pass "$conf: file validates against schema" || fail "$conf: $out"
    done
    python3 -I -c 'import json;json.load(open("'"$schema"'"))' && pass "the schema is valid JSON" || fail "schema is not JSON"

    _scn "the schema carries type, unit, range, enum, required, default and fallback"
    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-DEPLOY-001 --out "$T/v.yaml" --schema "$schema" >/dev/null 2>&1
    python3 -I - "$schema" <<'PY' && pass "constraints present" || fail "schema lacks constraints"
import json,sys
s=json.load(open(sys.argv[1]))
p=s["properties"]["parameters"]["properties"]
kv=p["FEAT-UAV-PROP"]["properties"]["motorKV"]
mode=p["FEAT-UAV-TELEM"]["properties"]["mode"]
cur=p["FEAT-UAV-PROP"]["properties"]["maxCurrentA"]
assert kv["type"]=="number" and kv["minimum"]==900 and kv["maximum"]==1200, kv
assert mode["enum"]==["eco","normal","sport"] and mode["type"]=="string", mode
assert "unit: A" in cur["description"] and cur["default"]==40 and cur["x-syscribe-fallback"]==30, cur
assert "motorKV" in p["FEAT-UAV-PROP"]["required"] and "mode" in p["FEAT-UAV-TELEM"]["required"]
assert p["FEAT-UAV-PROP"]["additionalProperties"] is False
PY

    _scn "the schema rejects a value outside the contract"
    sed 's/^\( *motorKV:\) 1000/\1 5000/' "$T/v.yaml" >/dev/null
    "$SYSCRIBE" -m "$U" params export --config CONF-UAV-DEPLOY-001 --out "$T/d.yaml" >/dev/null 2>&1
    sed -i 's/^\( *motorKV:\) 1000 /\1 5000 /' "$T/d.yaml"
    out=$(js "$T/d.yaml" "$schema") && fail "an out-of-range value validated" || pass "motorKV 5000 is rejected"
    rm -rf "$T"
}
