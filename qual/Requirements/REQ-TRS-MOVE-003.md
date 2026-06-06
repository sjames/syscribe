---
id: REQ-TRS-MOVE-003
type: Requirement
name: Move Is Atomic
title: Tool shall apply a move atomically (all-or-nothing)
status: draft
reqDomain: software
verificationMethod: test
---

The `move` command (REQ-TRS-MOVE-001) **shall** apply the file relocation and all reference updates atomically: either every change is applied, or — on any error — none is, leaving the model byte-for-byte as it was before the command ran.

In particular, the tool **shall** validate all preconditions and compute every planned write before mutating any file, and **shall** restore the original state (file contents and locations) if any write or the relocation itself fails.

**Source:** Feature request — "the move must be done atomically".

**Acceptance criteria:** when a precondition fails (e.g. the destination already exists), the command exits non-zero and no file on disk is modified, created, or removed.
