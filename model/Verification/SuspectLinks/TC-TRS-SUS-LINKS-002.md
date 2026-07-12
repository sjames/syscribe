---
type: TestCase
id: TC-TRS-SUS-LINKS-002
name: "Projection excludes editorial fields; body/normative changes alter the BLAKE3 hash"
status: active
testLevel: L1
sourceFile: repo:crates/syscribe/tests/suspect_links.rs
tags:
  - traceability
  - suspect-links
verifies:
  - REQ-TRS-SUS-LINKS-002
---

Verifies the content projection and BLAKE3 digest: changes confined to excluded
editorial/presentation fields do not change the hash, while changes to the body or a
normative frontmatter field do; and the stored value is algorithm-prefixed.

```gherkin
Feature: Content projection and BLAKE3 hashing

  Scenario: Editing an excluded field does not change the hash
    Given an element E with a computed projection hash H
    When only E's displayOrder (or extRef, or name, or layout) is changed
    Then the recomputed projection hash equals H

  Scenario: Editing the markdown body changes the hash
    Given an element E with a computed projection hash H
    When E's markdown body is edited
    Then the recomputed projection hash differs from H

  Scenario: Editing a normative field changes the hash
    Given an element E with a computed projection hash H
    When E's status (or reqDomain, or a SIL/ASIL field) is changed
    Then the recomputed projection hash differs from H

  Scenario: Cosmetic reformatting is canonicalized away
    Given two elements with identical projected content but different key order and line endings
    Then their projection hashes are equal

  Scenario: The digest is algorithm-prefixed
    Given a computed baseline for any target
    Then the stored value begins with "blake3:"
```
