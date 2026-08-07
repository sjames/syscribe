---
type: PlanningItem
id: PI-ERR-WAIVED-001
name: "Leaf marked done with only rationale-waived evidence"
status: done
itemType: task
achieves: REQ-ERR-DEMO-001
evidence:
  - ref: PI-DOES-NOT-EXIST-999
    rationale: "Tracked in an external system, not yet modeled here."
  - path: reports/not-written-yet.pdf
    rationale: "Report is planned but not written yet."
---

**Deliberate error demonstration 2 of 2.** A leaf `PlanningItem` at `status:
done` whose `evidence:` list is **non-empty but every entry carries its own
`rationale:`**, so nothing in the list actually counts as proof
(`REQ-TRS-PLANITEM-005`'s waiver excuses each entry's own *check* — it does
not manufacture a passing entry). `REQ-TRS-PLANITEM-006` still requires at
least one entry that genuinely resolves; a waived-only list never satisfies
that. Expect the same finding as `PI-ERR-NOEV-001`, for a different reason:

```
E719  leaf PlanningItem is `status: done` but has no non-waived, resolving `evidence:` entry
```

(Note: neither entry would resolve even without its `rationale:` — `PI-DOES-NOT-EXIST-999`
is not a real element and `reports/not-written-yet.pdf` does not exist on
disk — but that's incidental. The point of this fixture is specifically that
the waiver removes them from consideration regardless of whether they'd
otherwise resolve.)
