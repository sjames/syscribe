---
id: REQ-TRS-PARSE-011
type: Requirement
name: Tool shall attach an about comment file to every element it lists instead of creating a new element
status: draft
reqDomain: software
verificationMethod: test
---

Spec §3.10 defines **`about:` comment files** (the SysML v2 `comment … about
X, Y` construct): a file whose frontmatter lists, under `about:`, the
elements its Markdown body annotates. The body is a cross-element comment,
not documentation of an element of its own. `about:` is distinct from a
locale variant's `qualifiedName:` (REQ-TRS-PARSE-010): a variant is the
**documentation** of **one** element in another language, an `about:` file
is an additional **comment** on **one or more** elements. The tool **shall**:

1. treat every file other than a package `_index.md` whose frontmatter
   carries `about:` (a string or a list of strings) and which is not a
   locale variant as an `about:` comment: it **shall not** become an element
   (no path-derived qualified name, and no finding an element would raise);
2. resolve each `about:` entry like a cross-reference — against the
   qualified names of the model's elements (file-backed and synthesized),
   falling back to a stable `id:` — and attach the body, together with the
   comment's `name:`, optional `locale:` and source file, as a note to each
   resolved element (once per element);
3. report each entry that resolves to no element as error **`E027`**; when
   no entry resolves, the file is kept as its own element so it is not
   silently lost;
4. report warning **`W052`** when an `about:` comment declares frontmatter
   other than `type`, `name`, `about` and `locale` (a comment defines no
   element, so those fields are ignored), when an `about:` entry is not a
   non-empty string, or when a package `_index.md` carries `about:` (the
   `_index.md` defines its package and is never a comment; the field is
   ignored);
5. show each attached comment in `show` (a `## Note: <name>` section, the
   source file and locale, then the body, plus a `notes` row with the count).

A locale variant (a file with both `locale:` and `qualifiedName:`) that also
sets `about:` stays a locale variant; the `about:` field is one of the
fields it ignores (`W051`).

**Source:** GH issue #164; spec §3.10.

**Acceptance criteria:** with `Engine.md`, `Transmission.md` and a comment
file `SafetyNote.md` (`about: [VehicleSystem::Engine,
VehicleSystem::Transmission]`), validation raises no finding for
`SafetyNote.md`, no element `VehicleSystem::SafetyNote` exists, and `show`
of each target prints the comment body under `Note: SafetyNote`; a comment
entry naming a missing element raises `E027` naming it while the other
entries still attach; a comment whose only entry is missing raises `E027`
and is kept as an element; an extra structural field and `about:` on an
`_index.md` each raise `W052`.
