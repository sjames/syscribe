# From a chat to traced, tested code

A real session, not a mock-up: a Claude Code agent with the Syscribe MCP server connected took a one-paragraph
idea to a small, tested Python library (`src/ratelimit.py`, about 100 lines) and left behind the model that explains
it — the design discussion, ADRs, requirements, tests, and the work plan — all in this directory, all validating clean.

```bash
syscribe -m model/ validate                        # 0 errors, 0 warnings
syscribe -m model/ show REQ-RL-005                 # a requirement, its ADR, satisfier and verifying test
python3 -m venv .venv && .venv/bin/pip install pytest && .venv/bin/python -m pytest   # 17 passed
```

## The conversation

Nine messages, sent in order to one `claude -p` / `--continue` session. This is everything the human typed:

1. **Set up.** *"I want to build a small Python library that does rate limiting for API calls. The syscribe MCP server is connected and I want to keep the project's design, requirements, tests and work plan in a Syscribe model under ./model (an empty root package already exists). Set the model up for this project (id prefix RL) and tell me briefly what you set up and what we should talk about next. Don't invent design decisions yet."*
2. **Design discussion** (no model changes). *"It's for a single-process service, sync Python 3.10+, standard library only, called from multiple threads. Limits are per key (e.g. per API client). Callers should be able to either try and get rejected with a retry-after, or block until allowed. Before we write anything down: which algorithm should we use — fixed window, sliding window, or token bucket? Give me the trade-offs and a recommendation, but don't change the model yet."*
3. **Write the design.** *"It's to smooth out our own traffic and protect our own backend, so no strict downstream window. Token bucket it is. Please write the design down: the ADR for the algorithm choice (accepted) and a small software architecture — the main parts and how they relate. Keep it minimal, we're going to build a few hundred lines. Validate as you go and tell me what you created."*
4. **Requirements.** *"Yes, write the requirements. Start with a small stakeholder-level set and break them down into testable leaf requirements, with the breakdown traced back to the ADR, and link the architecture parts that satisfy them. Keep it lean — I'd rather have 6-8 good requirements than 20. Validate and tell me what the coverage looks like."*
5. **Tests.** *"Good. Now write the test cases for the leaf requirements, with Gherkin scenarios, and a test plan. Tests will be pytest functions in tests/test_ratelimit.py with a fake clock, and the library will be a module src/ratelimit.py. Decide the public API shape needed to write the tests (keep it small) and record that decision properly. Then check coverage again."*
6. **Plan.** *"Yes, add the L3 end-to-end test so the stakeholder requirements are covered too. Then plan the work: create the planning items in the model (an epic-level item tied to the requirements it achieves, with children for the concrete tasks and sensible blockedBy ordering). Show me the plan as a tree with status and what each item is blocked by."*
7. **Implement.** *"One global lock is fine for locking granularity — record that as an ADR. Then implement the plan: work through the planning items in dependency order, claiming each one as you start, writing the code and tests under src/ and tests/, running pytest, and marking items done with evidence only once the tests actually pass. Stop and tell me if anything in the model needs to change. At the end show me the plan status, coverage, and the validation result."*
8. **Unblock.** The agent stopped: `pytest` wasn't installed and it had no permission to install it. It said so, did not fake a result, and left the item `in_progress`. *"I've installed pytest into .venv (run it as .venv/bin/python -m pytest). Continue the plan from where you stopped, through to PI-RL-007. Claim each item as you start it, mark items done with evidence only once the tests actually pass, and stop if the model needs to change. At the end show me the plan status, coverage, and the validation result."*
9. **Close out.** *"There's no newer Python on this machine, so drop the newer-Python step from PI-RL-007 (note in the item that the suite was only run on the 3.10 minimum). Then close out PI-RL-007 and the epic if everything else is true, and give me the final plan tree, coverage and validation result. Also add test-results.xml to .gitignore."*

## What it produced

| Stage | Files |
|---|---|
| Design | `model/Design/ADR_RL_001.md` (token bucket, with rejected alternatives) … `ADR_RL_004.md` (one global lock); `RateLimiter`, `TokenBucket`, `Clock` parts |
| Requirements | `model/Requirements/REQ_RL_001..008.md`: 3 stakeholder, 5 leaf; every derived one cites its `breakdownAdr` |
| Tests | `model/Tests/TC_RL_001..006.md` with Gherkin (L1, L2 thread stress, L3 end-to-end) and `TP_RL_001.md` |
| Plan | `model/Plan/PI_RL_001..007.md`: an epic that `achieves` all eight requirements, six tasks, `blockedBy` ordering, evidence on every `done` |
| Code | `src/ratelimit.py`, `tests/test_ratelimit.py` (17 tests; function names match the model's `testFunctions`) |

The final state: 8 of 8 requirements verified, 7 of 7 plan items done, 0 validation errors and warnings.

## Things worth knowing before you trust it

This is the agent's output with the human steering, not a hand-polished example. Some of what you'll see is worth a reviewer's eye:

- **The agent made judgement calls.** The three-part architecture, the `Clock` seam, ADR-RL-002's content and the public API (ADR-RL-003) were its proposals, flagged as such in its replies. It also promoted requirements to `verified` and test cases to `active` itself, skipping a human `approved` sign-off.
- **A weak test was caught and strengthened.** The thread-safety test caught the race in only 4 of 40 runs against the deliberately unlocked code; the agent changed it to repeat the scenario, after which it caught the race every time.
- **Only Python 3.10 was exercised.** No newer interpreter was available, and PI-RL-007 says so.
- **Generated file names are qname-shaped** (`REQ_RL_005.md`, with `id: REQ-RL-005` inside), because the guarded-write `create_element` tool derives the path from a hyphen-free qualified name.
