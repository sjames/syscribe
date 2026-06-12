---
id: REQ-TRS-META-002
type: Requirement
name: Tool shall render applied stereotypes as «Name» banners in element diagrams
status: draft
reqDomain: software
verificationMethod: test
---

The diagrams **shall** surface the stereotypes an element applies ([[REQ-TRS-META-001]]) as
**«Name» banners** on the element's box, reusing the existing stereotype rendering (the
`stereotype_fg` styling and the `«…»` banner used for the type-keyword), so a stereotyped
element reads like a UML-style `«Critical»` block without SysMLv2 having UML stereotypes.

### Behaviour

- In the element / BDD diagrams, an element that applies one or more `MetadataDef`
  stereotypes **shall** show a **«Name»** banner per applied stereotype (the `MetadataDef`'s
  display name), in addition to any type-keyword banner.
- Banners **shall** use the shared diagram theme's stereotype styling and the shared font
  metrics (consistent with the other diagram text).
- Unresolved/invalid applications ([[REQ-TRS-META-001]] `E317`/`E318`) are a validation
  concern; the renderer shows what resolves and does not crash on a bad reference.

**Source:** companion of [[REQ-TRS-META-001]] (the model/validation/show/list core); split
out because it threads stereotype data through the diagram layout pipeline.

**Acceptance criteria:**

- An element applying `metadata: [Safety::Critical]` renders a `«Critical»` banner in its
  diagram box.
- An element applying two stereotypes shows both banners.
- An element with no applied stereotype renders unchanged (only its type-keyword banner, if
  any).
