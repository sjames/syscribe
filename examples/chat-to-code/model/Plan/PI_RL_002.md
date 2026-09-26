---
type: PlanningItem
id: PI-RL-002
itemType: task
name: Scaffold the project
parent: PI-RL-001
status: done
assignedTo: claude
evidence:
- path: ../pyproject.toml
- path: ../src/ratelimit.py
---

Create the package layout: `src/ratelimit.py` (empty module), `tests/`, and `pyproject.toml` declaring `requires-python = ">=3.10"` and pytest as a development dependency only.

Enables the `requires-python` check in TC-RL-005 (REQ-RL-008).

