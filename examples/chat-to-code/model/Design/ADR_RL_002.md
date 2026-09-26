---
type: ADR
id: ADR-RL-002
name: Target a single-process, synchronous, multi-threaded, stdlib-only environment
status: accepted
---

## Context

The library is for a single-process service written in synchronous Python, called from multiple threads. Limits are per key (for example, per API client).

## Decision

- Single process only: no cross-process or distributed limiting.
- Synchronous API only.
- Python 3.10 or later.
- Standard library only: no third-party dependencies.
- Safe to call from multiple threads.

## Consequences

- No shared external store or async support to maintain.
- The library must be internally thread-safe (REQ-RL-007).
- All functionality must be built from the standard library (REQ-RL-008).

