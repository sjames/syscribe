---
type: Requirement
id: REQ-TRS-PKG-000
name: "A package's membership is always presented from the directory, so it cannot drift from the model"
status: draft
reqDomain: software
reqClass: stakeholder
tags:
  - packages
---

Syscribe shall present every package's membership from the directory tree that defines it, and
shall steer authors away from hand-maintained member lists in `_index.md` prose, so that package
overviews never disagree with the files on disk.
