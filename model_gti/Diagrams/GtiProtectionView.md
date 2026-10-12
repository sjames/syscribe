---
type: Diagram
name: GtiProtectionView
diagramKind: IBD
subject: System::GridTieInverter
include: {kinds: [protection]}
style: safety
---

The protection path: the hardware trips from the control board to the power stage and the AC relays. Every edge that is not `flowKind: protection` is faded; the `safety` style draws the trip lines heavier. Changing `style: safety` to `style: isolate` leaves the rest out instead of fading it.
