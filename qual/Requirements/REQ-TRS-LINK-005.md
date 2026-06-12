---
id: REQ-TRS-LINK-005
type: Requirement
title: Live server shall show a per-element source-link icon to the hosted model element
status: draft
reqDomain: software
verificationMethod: test
---

In the **live web UI**, the tool **shall** show a small **source-link icon** that opens an
element's hosted URL ([[REQ-TRS-LINK-001]]) in a new tab — **distinct** from the existing
in-app navigation, so clicking a diagram shape still navigates internally while the icon takes
the user to the hosted source.

### Behaviour

- The element **detail panel** **shall** render a small "view source" icon/link (e.g. next to
  the element's title) pointing at the element's resolved URL, `target="_blank"
  rel="noopener"`.
- Diagram shapes in the live view keep their existing internal-navigation behaviour; the
  external link is the **icon**, not the whole shape (so the two affordances do not collide).
- The icon is shown **only** when `[links]` is configured **and** the element resolves to a URL;
  otherwise it is absent.
- This is presentation-only — it never changes model data or validation.

**Source:** a "view source on the host" affordance in the live server, alongside its internal
navigation. Consumes [[REQ-TRS-LINK-001]]; complements the exported-SVG links of
[[REQ-TRS-LINK-002]].

**Acceptance criteria:**

- With `[links]` configured, the element detail panel shows a source-link icon whose `href` is
  the element's resolved hosted URL and opens in a new tab.
- The icon is absent for an element with no resolved URL, and absent entirely when `[links]` is
  not configured.
- Clicking a diagram shape still performs in-app navigation (the external link is the icon only).
