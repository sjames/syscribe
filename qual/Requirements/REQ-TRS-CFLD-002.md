---
id: REQ-TRS-CFLD-002
type: Requirement
title: Tool shall filter elements by a --where predicate over custom fields
status: draft
reqDomain: software
verificationMethod: test
---

The tool **shall** provide a generic `--where` predicate that filters elements by their
`custom_fields:` ([[REQ-TRS-CFLD-001]]) values, mirroring the existing `list`/`find`
filter chain.

### `--where <predicate>`

The lens flag **shall** be accepted on the element-listing commands `ls` and `find`. Its
argument addresses a custom field via the `custom.<key>` namespace and supports these
operators:

| Form | Meaning |
|---|---|
| `custom.<key>=<value>` | **exact** match (the field equals the value; for a list field, any element equals) |
| `custom.<key>=~<pattern>` | **substring / regex** match against the field's string form |
| `custom.<key>~=<value>` | **list membership** (the list field contains the value) |
| `custom.<key>` | **presence** (the field is set, any value) |

- An element matches `--where` only if it declares the named custom field and the
  operator holds; elements without the field are excluded (except this is the
  definition of the presence form failing).
- `--where` **shall compose** with the existing `ls`/`find` filters (type, tag, status):
  all supplied filters apply (AND).
- An unparseable `--where` argument (unknown operator spelling) **shall** be a usage
  error with a clear message and non-zero exit.
- `--where` **shall** be deterministic and add no findings (it is a read-only query).

**Source:** GH #39 (custom fields query).

**Acceptance criteria:**

- `ls --where custom.supplier=Bosch` lists exactly the elements whose
  `custom_fields.supplier` equals `Bosch`; `find --where custom.maturity=prototype`
  likewise.
- `ls --where custom.costCenter=~PWT` matches by substring/regex; `ls --where
  custom.partNumbers~=A-1001` matches a list field containing the value; `ls --where
  custom.supplier` matches any element that declares that field.
- `--where` composes with a type filter (e.g. `ls PartDef --where custom.supplier=Bosch`)
  and an unparseable predicate exits non-zero.
