---
type: Requirement
id: REQ-TRS-VIS-022
name: "The Allocation generator derives logical-to-physical allocation maps with swimlanes from a Package, AllocationDef or Allocation subject"
status: draft
reqDomain: software
reqClass: system
derivedFrom: [REQ-TRS-VIS-015]
breakdownAdr: Decisions::VisualisationADR
tags:
  - diagram
  - visualisation
  - allocation
---

For a derived `diagramKind: Allocation` whose subject is a `Package` (or `LibraryPackage`/
`Namespace`), `AllocationDef` or `Allocation`, the generator shall collect every allocation pair
under the subject: an `Allocation` element's top-level `allocatedFrom:`/`allocatedTo:`, each
`features:` entry of `type: Allocation` with those two fields, each `allocations:` entry of an
`AllocationDef`, and each `allocatedTo:` on a `Part`/`PartDef`/`Action`/`ActionDef` under the
subject (its source being the element itself). It shall produce:

- two `Swimlane` nodes, `logical` and `physical`, containing one `Block` node per distinct source
  and per distinct target element respectively, each drawn with its real type's stereotype (an
  unresolved end becomes a dashed block labelled by the reference text);
- one `Allocation` edge per pair from the source block to the target block, labelled by the
  allocation usage's `name` when present, with the `«allocate»` keyword from `vis::style`.

`include:`/`exclude:` apply to the end elements. Shape ids are `derived_shape_id(<element
qualified name>)`; the lanes are `<subjectId>-logical` and `<subjectId>-physical`. Layout hints:
layered, left-to-right, hierarchical.

## Rationale

§12.6's HW/SW independence rule and `E314` make allocation the one relationship a safety
reviewer must see; the demo model's `FunctionAllocation` has four pairs no diagram shows.

## Scope

- An element that appears as both a source and a target is drawn once in each lane.
- Allocation of requirements (`RequirementAllocation`) is included when its ends resolve.
