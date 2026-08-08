tc_TRS_SYSMLV2_008() {
    local F="$1"; local M="$F/TC-TRS-SYSMLV2-008/model"

    _scn() { SCENARIO_NAME="$1"; printf "  ▶ %s\n" "$1"; }

    # 1. all four annotations lift onto a part def
    _scn "all four annotations lift onto a part def"
    local out; out=$("$SYSCRIBE" -m "$M" export 2>/dev/null)
    local qn="SysML2::Demo::AllFieldsPart"
    local domain; domain=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.domain')
    [ "$domain" = "software" ] && pass "domain lifted to software" || fail "domain='$domain' (expected software)"
    local asil; asil=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.asilLevel')
    [ "$asil" = "B" ] && pass "asilLevel lifted to B" || fail "asilLevel='$asil' (expected B)"
    local sn; sn=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.shortName')
    [ "$sn" = "all-fields-part" ] && pass "shortName lifted to all-fields-part" || fail "shortName='$sn' (expected all-fields-part)"
    local ib; ib=$(printf '%s' "$out" | jq -r --arg q "$qn" '.elements[] | select(.qname==$q) | .frontmatter.implementedBy[0]')
    [ "$ib" = "services/all-fields-part/" ] && pass "implementedBy lifted to services/all-fields-part/" \
        || fail "implementedBy='$ib' (expected services/all-fields-part/)"

    # 2. @SyscribeIntegrity with both asil and sil raises the existing W006
    _scn "@SyscribeIntegrity with both asil and sil raises the existing W006 mutual-exclusion warning"
    local vout; vout=$("$SYSCRIBE" -m "$M" validate 2>&1 || true)
    local w006_count; w006_count=$(printf '%s' "$vout" | grep -c 'W006' || true)
    [ "$w006_count" -eq 1 ] && pass "exactly one W006 raised" || fail "W006 count=$w006_count (expected 1)"

    # 2b. a @SyscribeDomain-lifted domain genuinely drives the existing E313
    # domain-compatibility check against a satisfy target's reqDomain — not
    # just a copied value nobody downstream reads. (MismatchedDomainPart is
    # the only `satisfy` in this fixture, so a bare E313/REQ-DOMAIN-001 check
    # is unambiguous — the finding message itself doesn't name the element.)
    _scn "a @SyscribeDomain-lifted domain mismatch against satisfy target's reqDomain raises the existing E313"
    printf '%s' "$vout" | grep 'E313' | grep -q 'REQ-DOMAIN-001' \
        && pass "E313 raised for the domain mismatch against REQ-DOMAIN-001" \
        || fail "E313 not raised for the domain mismatch against REQ-DOMAIN-001"

    # 2c. a @SyscribeImplementedBy path that doesn't exist on disk raises the
    # existing W023 disk-check. (AllFieldsPart is the only implementedBy in
    # this fixture, so a bare W023 check is unambiguous.)
    _scn "a @SyscribeImplementedBy-lifted path that doesn't exist on disk raises the existing W023"
    printf '%s' "$vout" | grep -q 'W023' \
        && pass "W023 raised for the lifted implementedBy path" || fail "W023 not raised for the lifted implementedBy path"

    # 3. a part def with no annotation carries no lifted fields
    _scn "a part def with no annotation carries no lifted fields"
    local plain; plain=$(printf '%s' "$out" | jq -c --arg q "SysML2::Demo::PlainPart" \
        '.elements[] | select(.qname==$q) | .frontmatter | {domain, asilLevel, silLevel, plLevel, shortName, implementedBy}')
    local nonnull; nonnull=$(printf '%s' "$plain" | jq '[.[] | select(. != null)] | length')
    [ "$nonnull" -eq 0 ] && pass "PlainPart carries no lifted fields" || fail "PlainPart unexpectedly carries lifted fields: $plain"
}
