---
type: Requirement
id: REQ-TRS-SIDECAR-001
name: "The mutation and campaign sidecars are bounded, safe to read and written atomically"
status: draft
reqDomain: software
reqClass: system
tags:
  - cli
  - evidence
---

`.syscribe/mutation.json` (`mutate --record`) and `.syscribe/campaigns.json` (`inject --record`) shall be bounded, shall never be overwritten when they cannot be read, and shall be written so an interruption cannot truncate them (GH #472, #473, #474).

- **Bounded.** `campaigns.json` keeps the latest `HISTORY` (20) records per campaign key. `mutation.json` keeps the newest `MUTATION_KEEP` (100) records per machine **and subject**. The bound is per subject and generous because a machine has one record per hash and one projected onto many Configurations hashes differently for each; an evicted record is a score that reads as stale, not lost evidence.
- **Unreadable is not empty.** A missing file is no records. A file that exists but cannot be read or parsed is an error naming it: `mutate --record` and `inject --record` refuse to overwrite it (exit 1, saying to fix or remove it), and `requirement-coverage` / `exec-coverage` report it in the evidence `errors` instead of silently using no scores (a refused `mutate --record` is not the `--min-score` gate, exit 2).
- **Atomic.** Both are written beside the file and renamed over it (`fsio::write_atomic`), so the policy guard and `--dry-run` apply to the file itself and an interruption leaves the previous file or the new one, never a truncated one.
