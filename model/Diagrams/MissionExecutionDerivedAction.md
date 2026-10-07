---
type: Diagram
name: MissionExecutionDerivedAction
diagramKind: Action
subject: Behavior::MissionExecution
---

Action diagram **derived from the model** (`REQ-TRS-VIS-019`): the `takeoff`, `navigate` and `land`
perform-actions of `Behavior::MissionExecution`, the `checkWeather` conditional drawn as a decision
diamond with its `abortMission`/`continueMission` branches rejoining at a merge, the `missionStart`
fork and `missionEnd` join control nodes, and the `successionConnections:` ordering from an initial
node to a final one — all read from the `ActionDef`'s own frontmatter.
