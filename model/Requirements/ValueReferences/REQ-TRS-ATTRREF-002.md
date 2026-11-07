---
type: Requirement
id: REQ-TRS-ATTRREF-002
name: "An unresolvable value reference is E331 and names the available attributes"
status: draft
reqDomain: software
reqClass: system
tags:
  - references
  - attributes
---

`validate` shall report `E331` for a `{{Elem.attr}}` value reference (GH #452):

1. {#element} **Element.** whose element part resolves to no element, or whose basic name several
   elements share (the message lists their qualified names) (an unresolved bare name that
   starts with a lowercase letter, such as `{{ user.name }}`, is template text and is not checked);
2. {#attribute} **Attribute.** whose attribute path names no attribute of the element; the message
   lists the attributes that have a value, so a wrong guess self-corrects;
3. {#scalar} **Scalar.** whose attribute exists but holds no single scalar value (a list, a map, or a
   feature with no `value:`/`default:`).
