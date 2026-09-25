---
id: REQ-TRS-PARSE-010
type: Requirement
name: Tool shall attach a locale documentation-variant file to its base element instead of creating a new element
status: draft
reqDomain: software
verificationMethod: test
---

Spec §3.10 defines **locale documentation variants**: a file carrying both
`locale:` (a BCP 47 tag) and `qualifiedName:` (the qualified name of an
existing element) contributes a locale-tagged documentation body to that
element and does not define a model element of its own. The tool **shall**:

1. treat every file with **both** `locale:` and `qualifiedName:` as a locale
   variant: it **shall not** become an element (so it gets no path-derived
   qualified name such as `VehicleSystem::Engine.de`, and raises no finding
   that an element would, e.g. `W042`);
2. resolve `qualifiedName:` against the qualified names of the file-backed
   elements and attach the variant's Markdown body to the target as its
   documentation for that locale, collected as a map `locale → doc`;
3. report a variant whose `qualifiedName:` resolves to no element as error
   **`E026`**; the file is then kept as its own element (the pre-existing
   behaviour) so it is not silently dropped;
4. report warning **`W051`** when a variant attaches to an element that
   already has documentation for that locale (another variant, or the
   element's own `locale:`) — the first file in walk order wins; when the
   variant's `type:` differs from the target's type; or when the variant
   declares frontmatter other than `type`, `name`, `locale` and
   `qualifiedName` — a variant contributes documentation only and never
   redefines the element's structure, so those fields are ignored;
5. show each attached locale body in `show` (a `## Documentation (<locale>)`
   section, plus a `localeDocs` row listing the attached locales).

A file with `locale:` but no `qualifiedName:` is an ordinary element whose own
body is tagged with that locale.

**Source:** GH issue #160; spec §3.10.

**Acceptance criteria:** with `Engine.md` and a German variant `Engine.de.md`
(`qualifiedName: VehicleSystem::Engine`, `locale: de`), validation raises no
`W042` and no element `VehicleSystem::Engine.de` exists; `show
VehicleSystem::Engine` prints the German body under `Documentation (de)`; a
variant naming a missing element raises `E026`; a second `de` variant for the
same element and a variant with an extra structural field each raise `W051`.
