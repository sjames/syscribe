---
id: REQ-TRS-NAME-001
type: Requirement
name: Tool shall flag element name segments that are not SysMLv2 basic names
status: draft
reqDomain: software
verificationMethod: test
---

Element names (and the path-derived qualified-name segments that reference them)
**shall** follow the SysMLv2 **basic-name** grammar so that every name is usable in
every reference context — including the tokenized expression contexts (`appliesWhen`,
`parameterConstraints`) where a hyphen is reserved as the subtraction operator.

### Basic name

- A basic name matches `^[A-Za-z_][A-Za-z0-9_]*$` (a letter or `_`, then letters,
  digits or `_`). This is the SysMLv2 basic-identifier grammar; hyphens, spaces and
  other punctuation are **not** permitted.

### Validation

- The basic-name rule applies to **every** qualified-name segment — both an
  element's **own name** and the **package / directory (namespace) segments** above
  it — because each is a referenceable segment.
- For each element, the tool **shall** check the element's **own name** (the final
  `::`-segment of its qualified name). A segment that is **neither** a basic name
  **nor** a stable id (`REQ-*`, `TC-*`, `TP-*`, `ADR-*`, and the safety/security ids,
  which legitimately contain `-`) **shall** raise warning **`W042`**, naming the
  segment and recommending `_` or CamelCase (e.g. `Anti-Lock` → `Anti_Lock` /
  `AntiLock`).
- A **package / directory** name **shall** likewise be checked. A package declared
  with an `_index.md` is covered by the own-name check above (the `_index` element's
  own name is the directory name). A directory **without** an `_index.md` owns no
  element, so the tool **shall** additionally check each ancestor (namespace)
  segment and raise `W042` once per distinct non-conforming directory name,
  attributed to the directory.
- `W042` is a **warning** (advisory, gateable with `--deny W042`) so existing models
  with non-conforming names have a migration path rather than a hard break.
- The existing `E209` (invalid `appliesWhen` expression) continues to reject a
  hyphenated reference such as `Features::Anti-Lock`; its message **shall** hint that
  names must be basic identifiers. The two findings are complementary: `W042` guides
  the fix at the name's definition, `E209` at the reference.

### Not in scope (v1)

- SysMLv2 **unrestricted names** (single-quoted, e.g. `Features::'Anti-Lock'`) are
  **not** supported; the convention is basic names only.

**Source:** GH #42; SysMLv2/KerML basic-name grammar; CLAUDE.md §ID Scheme.

**Acceptance criteria:**

- A `FeatureDef` (or any non-stable-id element) whose name segment is `Anti-Lock`
  raises `W042` naming the segment; the same element renamed `Anti_Lock` or
  `AntiLock` raises no `W042`.
- A stable-id-named element (e.g. file `REQ-PWR-001.md`) raises **no** `W042` despite
  the hyphens in its id — only its **containing directories** must be basic names.
- A hyphenated **directory** name (both an `_index.md` package and a directory
  without one) raises `W042`, attributed to the directory; a basic directory name
  (`Good_Pkg`) does not.
- A hyphenated `appliesWhen` reference (`Features::Anti-Lock`) still raises `E209`.
- A model whose names are all basic (or stable ids) produces no `W042`.
