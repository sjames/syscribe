"""Tests for ratelimit, traced to TC-RL-001 .. TC-RL-006 in the Syscribe model."""

import ast
import re
import sys
import threading
import time
from pathlib import Path

import pytest

from ratelimit import RateLimiter

ROOT = Path(__file__).resolve().parents[1]


class FakeClock:
    """Manually driven clock; sleep() records the request and advances time."""

    def __init__(self, start=0.0):
        self._now = start
        self.sleeps = []

    def now(self):
        return self._now

    def advance(self, seconds):
        self._now += seconds

    def sleep(self, seconds):
        self.sleeps.append(seconds)
        self._now += seconds


def drain(limiter, key):
    """Call try_acquire until refused; return how many calls were allowed."""
    allowed = 0
    while limiter.try_acquire(key).allowed:
        allowed += 1
        assert allowed < 10_000, "limiter never refused"
    return allowed


# --- TC-RL-001: per-key token bucket admission and refill (REQ-RL-004) -------


def test_admits_up_to_capacity_then_refuses():
    limiter = RateLimiter(rate=1, capacity=3, clock=FakeClock())
    results = [limiter.try_acquire("a").allowed for _ in range(4)]
    assert results == [True, True, True, False]


def test_keys_are_independent():
    limiter = RateLimiter(rate=1, capacity=3, clock=FakeClock())
    assert drain(limiter, "a") == 3
    assert limiter.try_acquire("b").allowed


def test_refills_with_elapsed_time_up_to_capacity():
    clock = FakeClock()
    limiter = RateLimiter(rate=1, capacity=3, clock=clock)
    drain(limiter, "a")
    clock.advance(2)
    assert drain(limiter, "a") == 2
    clock.advance(100)
    assert drain(limiter, "a") == 3


# --- TC-RL-002: retry-after on refusal (REQ-RL-005) --------------------------


def test_refused_call_reports_retry_after():
    limiter = RateLimiter(rate=2, capacity=1, clock=FakeClock())
    drain(limiter, "a")
    decision = limiter.try_acquire("a")
    assert not decision.allowed
    assert decision.retry_after == 0.5


def test_retry_after_shrinks_as_time_passes():
    clock = FakeClock()
    limiter = RateLimiter(rate=2, capacity=1, clock=clock)
    drain(limiter, "a")
    clock.advance(0.25)
    decision = limiter.try_acquire("a")
    assert not decision.allowed
    assert decision.retry_after == 0.25


def test_admitted_after_retry_after_elapses():
    clock = FakeClock()
    limiter = RateLimiter(rate=2, capacity=1, clock=clock)
    drain(limiter, "a")
    refused = limiter.try_acquire("a")
    assert not refused.allowed
    clock.advance(refused.retry_after)
    assert limiter.try_acquire("a").allowed


def test_admitted_call_has_zero_retry_after():
    limiter = RateLimiter(rate=2, capacity=1, clock=FakeClock())
    decision = limiter.try_acquire("a")
    assert decision.allowed
    assert decision.retry_after == 0.0


# --- TC-RL-003: blocking acquire (REQ-RL-006) --------------------------------


def test_acquire_does_not_sleep_when_token_available():
    clock = FakeClock()
    limiter = RateLimiter(rate=2, capacity=1, clock=clock)
    limiter.acquire("a")
    assert clock.sleeps == []


def test_acquire_sleeps_once_for_retry_after():
    clock = FakeClock()
    limiter = RateLimiter(rate=2, capacity=1, clock=clock)
    drain(limiter, "a")
    limiter.acquire("a")
    assert clock.sleeps == [0.5]


def test_acquire_consumes_the_token_it_waited_for():
    clock = FakeClock()
    limiter = RateLimiter(rate=2, capacity=1, clock=clock)
    drain(limiter, "a")
    limiter.acquire("a")
    decision = limiter.try_acquire("a")
    assert not decision.allowed
    assert decision.retry_after == 0.5


# --- TC-RL-004: concurrent calls never exceed the bucket (REQ-RL-007) --------


@pytest.fixture
def aggressive_thread_switching():
    """Switch threads very often so a missing lock is likely to show up."""
    old = sys.getswitchinterval()
    sys.setswitchinterval(1e-6)
    yield
    sys.setswitchinterval(old)


def test_concurrent_callers_never_exceed_capacity(aggressive_thread_switching):
    n_threads, capacity, rounds = 200, 50, 30

    # A missing lock only loses a token occasionally, so the scenario is
    # repeated on fresh limiters to make such a race very likely to be seen.
    for _ in range(rounds):
        limiter = RateLimiter(rate=1, capacity=capacity, clock=FakeClock())  # frozen
        barrier = threading.Barrier(n_threads)
        results = []  # list.append is atomic

        def worker():
            barrier.wait()
            results.append(limiter.try_acquire("a").allowed)

        threads = [threading.Thread(target=worker) for _ in range(n_threads)]
        for t in threads:
            t.start()
        for t in threads:
            t.join()

        assert len(results) == n_threads
        assert sum(results) == capacity


# --- TC-RL-005: synchronous, Python 3.10+, stdlib only (REQ-RL-008) ----------


def _parse_module():
    return ast.parse((ROOT / "src" / "ratelimit.py").read_text())


def test_imports_are_stdlib_only():
    imported = set()
    for node in ast.walk(_parse_module()):
        if isinstance(node, ast.Import):
            imported.update(alias.name.split(".")[0] for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            assert node.level == 0, "relative imports are not expected"
            imported.add(node.module.split(".")[0])
    assert imported <= set(sys.stdlib_module_names), imported - set(sys.stdlib_module_names)


def test_api_is_synchronous():
    coroutine_nodes = (ast.AsyncFunctionDef, ast.Await, ast.AsyncFor, ast.AsyncWith)
    offenders = [n for n in ast.walk(_parse_module()) if isinstance(n, coroutine_nodes)]
    assert offenders == []


def test_declares_python_3_10_minimum():
    pyproject = (ROOT / "pyproject.toml").read_text()
    assert re.search(r'^requires-python\s*=\s*">=3\.10"\s*$', pyproject, re.MULTILINE)


# --- TC-RL-006: end to end on the real clock (REQ-RL-001, 002, 003) ----------
# Only lower bounds on elapsed time are asserted: sleeping may return late,
# never early, so these tests cannot flake on a slow machine.


def test_e2e_refused_key_gets_retry_after_and_other_key_unaffected():
    limiter = RateLimiter(rate=1, capacity=2)
    assert limiter.try_acquire("a").allowed
    assert limiter.try_acquire("a").allowed
    refused = limiter.try_acquire("a")
    assert not refused.allowed
    assert refused.retry_after > 0
    assert limiter.try_acquire("b").allowed


def test_e2e_blocking_caller_waits_on_real_clock():
    limiter = RateLimiter(rate=20, capacity=1)
    assert limiter.try_acquire("a").allowed
    start = time.monotonic()
    limiter.acquire("a")
    assert time.monotonic() - start >= 0.04


def test_e2e_threads_are_held_to_the_sustained_rate():
    n_threads, calls_each = 8, 10
    limiter = RateLimiter(rate=200, capacity=1)
    completed = []  # list.append is atomic

    def worker():
        for _ in range(calls_each):
            limiter.acquire("a")
            completed.append(1)

    threads = [threading.Thread(target=worker) for _ in range(n_threads)]
    start = time.monotonic()
    for t in threads:
        t.start()
    for t in threads:
        t.join()
    elapsed = time.monotonic() - start

    assert len(completed) == n_threads * calls_each
    # 80 calls, one token available at the start, then 200 tokens/s: >= 79/200 s.
    assert elapsed >= 0.35
