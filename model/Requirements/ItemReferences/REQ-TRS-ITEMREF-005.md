---
type: Requirement
id: REQ-TRS-ITEMREF-005
name: "Suspect links can be baselined at requirement-item granularity"
status: draft
reqDomain: software
reqClass: system
tags:
  - requirements
  - references
  - suspect-links
---

A change to one requirement item shall mark only the elements that rely on that item (GH #453).

1. {#key} **Key.** A `traceBaselines:` key may be `REQ-X#slug`; its value is the BLAKE3 hash of that item's text alone, without the anchor token and the item's number.
2. {#links} **Links.** Each `REQ-X#slug` citation in a source's body is an item-level trace link (kind `cites`), listed by `suspect list` and captured by `suspect accept <src> REQ-X#slug` or `--all-unbaselined`.
3. {#suspect} **Suspect.** Validation compares the stored hash with the item's current hash and raises the existing `W090` on a mismatch. Editing, inserting or reordering other items does not.
4. {#by-number} **By number.** A by-number reference (`REQ-X#5`, `REQ-X item 5`) or a question reference cannot be baselined; `suspect accept` refuses it and says to add an anchor.
