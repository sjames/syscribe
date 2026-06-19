---
type: Requirement
id: REQ-TRS-PUML-032
name: "E403: unrecognized pumlMode value"
status: draft
reqDomain: software
reqClass: derived
derivedFrom: [REQ-TRS-PUML-001]
breakdownAdr: Decisions::PlantUMLADR
tags:
  - diagram
  - plantuml
  - validation
---

If `pumlMode:` is set to any value other than `companion`, the validator shall emit
error **E403**.

## Recognised values

Currently `companion` is the only recognised value for `pumlMode:`. This mirrors the
design of `svgMode:`, which likewise accepts only `companion` as its single valid value.

## Error message

```
unrecognized `pumlMode` value '<value>' — only `companion` is supported
```

where `<value>` is the literal string supplied by the author.

## Severity rationale

This is an error (not a warning) because an unrecognised value indicates either a typo or
an intent to use an unsupported mode. In either case the `pumlMode:` field will silently
produce no output, and the author may not realise the companion pipeline has not been
activated. Failing hard surfaces the problem immediately.

## Forward-compatibility note

When additional `pumlMode` values are introduced in future (e.g. `inline`), this
requirement shall be updated to extend the recognised-values list and to remove those
values from the E403 trigger.
