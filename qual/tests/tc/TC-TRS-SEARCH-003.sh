tc_TRS_SEARCH_003() {
    # Fixture: Comms::REQ-S3-001..003 share "radio"/"link" vocabulary; Wheels::REQ-S3-010
    # (hydraulic caliper pressure) shares no content word with them.
    local F="$1"; local M="$F/TC-TRS-SEARCH-003/model" out rc
    local V="$F/TC-TRS-SEARCH-001/model"   # variant model (Sat-gated REQ-S1-012) for the config lens
    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }
    _jq() { printf '%s' "$3" | jq -e "$2" >/dev/null 2>&1 && pass "$1" || fail "$1 — got: $(printf '%s' "$3" | head -c 600)"; }
    local init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"qual","version":"1"}}}'
    local ALL='["REQ-S3-001","REQ-S3-002","REQ-S3-003","REQ-S3-010"]'

    _scn "k clusters partition the elements"
    local c2; c2=$("$SYSCRIBE" -m "$M" clusters --k 2 --json 2>/dev/null) || true
    _jq "k = 2 and 2 clusters" '.k==2 and (.clusters|length)==2' "$c2"
    _jq "every element appears in exactly one cluster" "([.clusters[].members[]]|sort)==$ALL" "$c2"
    _jq "cluster sizes sum to 4 clustered elements, size == member count" '([.clusters[].size]|add)==4 and all(.clusters[]; .size==(.members|length))' "$c2"
    _jq "each cluster carries a non-empty term label and member ids" 'all(.clusters[]; (.label|type)=="array" and (.label|length)>0 and all(.label[]; type=="string") and (.members|length)>0)' "$c2"
    _jq "labels name each cluster's vocabulary (radio / caliper)" '[.clusters[]|.label[0]]|sort==["caliper","radio"]' "$c2"

    _scn "Clustering is deterministic"
    local c2b; c2b=$("$SYSCRIBE" -m "$M" clusters --k 2 --json 2>/dev/null) || true
    [ -n "$c2" ] && [ "$c2" = "$c2b" ] && pass "two --json runs byte-identical" || fail "--json runs differ"
    local t1 t2
    t1=$("$SYSCRIBE" -m "$M" clusters --k 2 2>/dev/null) || true
    t2=$("$SYSCRIBE" -m "$M" clusters --k 2 2>/dev/null) || true
    [ -n "$t1" ] && [ "$t1" = "$t2" ] && pass "two text runs identical" || fail "text runs differ"
    printf '%s' "$t1" | grep -q '^## Cluster 1 \[caliper' && pass "text output lists the caliper cluster" || fail "text output: $t1"

    _scn "Cosine similarity groups by shared vocabulary"
    _jq "disjoint-vocabulary REQ-S3-010 is alone in its cluster" '.clusters[]|select(.members|index("REQ-S3-010"))|.members==["REQ-S3-010"]' "$c2"
    _jq "REQ-S3-010 does not share a cluster with lexically dissimilar REQ-S3-001" '[.clusters[]|select(.members|index("REQ-S3-010"))|.members[]]|index("REQ-S3-001")==null' "$c2"
    _jq "the three radio-link requirements share one cluster" '.clusters[]|select(.members|index("REQ-S3-001"))|.members==["REQ-S3-001","REQ-S3-002","REQ-S3-003"]' "$c2"

    _scn "--k is validated and clamped"
    "$SYSCRIBE" -m "$M" clusters --k 0 >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--k 0 exits 1" || fail "--k 0 exit $rc (expected 1)"
    out=$("$SYSCRIBE" -m "$M" clusters --k 9 --json 2>/dev/null) || true
    _jq "--k 9 clamped to the element count: k = 4, 4 clusters" '.k==4 and (.clusters|length)==4' "$out"
    _jq "clamped run still partitions all 4 elements (one per cluster)" "([.clusters[].members[]]|sort)==$ALL and all(.clusters[]; .size==1)" "$out"
    out=$("$SYSCRIBE" -m "$M" clusters --json 2>/dev/null) || true
    _jq "default k = min(8, 4) = 4" '.k==4' "$out"

    _scn "(title: config lens) --config projects before clustering"
    out=$("$SYSCRIBE" -m "$V" clusters --k 2 --json 2>/dev/null) || true
    _jq "unprojected variant model clusters REQ-S1-012" '[.clusters[].members[]]|index("REQ-S1-012")!=null' "$out"
    out=$("$SYSCRIBE" -m "$V" clusters --k 2 --config CONF-S1-001 --json 2>/dev/null) || true
    _jq "--config CONF-S1-001: gated REQ-S1-012 not clustered, 6 elements remain" '([.clusters[].members[]]|index("REQ-S1-012")==null) and ([.clusters[].size]|add)==6' "$out"
    "$SYSCRIBE" -m "$V" clusters --config bogus >/dev/null 2>&1 && rc=0 || rc=$?
    [ "$rc" -eq 1 ] && pass "--config bogus on the variant model exits 1" || fail "--config bogus exit $rc (expected 1)"

    _scn "The MCP clusters tool matches the CLI"
    out=$(printf '%s\n' "$init" \
        '{"jsonrpc":"2.0","id":2,"method":"tools/list"}' \
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"clusters","arguments":{"k":2}}}' \
        | timeout 20 "$SYSCRIBE" -m "$M" mcp 2>/dev/null) || true
    printf '%s' "$out" | jq -se '[.[]|select(.id==2)|.result.tools[]|select(.name=="clusters")][0].annotations.readOnlyHint==true' >/dev/null 2>&1 \
        && pass "clusters tool advertised with readOnlyHint true" || fail "clusters tool not advertised read-only"
    local tool cli
    tool=$(printf '%s' "$out" | jq -S 'select(.id==3)|.result.content[0].text|fromjson' 2>/dev/null) || true
    cli=$(printf '%s' "$c2" | jq -S . 2>/dev/null) || true
    [ -n "$tool" ] && [ "$tool" = "$cli" ] && pass "MCP clusters(k=2) document == clusters --k 2 --json" || fail "MCP clusters differs: $(printf '%s' "$tool" | head -c 400)"
}
