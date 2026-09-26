"""Token-bucket rate limiting for API calls."""

import threading
import time
from dataclasses import dataclass
from typing import Protocol

__all__ = ["Clock", "Decision", "RateLimiter", "SystemClock"]


class Clock(Protocol):
    """Source of time for the limiter."""

    def now(self) -> float:
        """Return monotonic time in seconds."""

    def sleep(self, seconds: float) -> None:
        """Wait for `seconds`."""


class SystemClock:
    """Clock backed by the real monotonic clock."""

    def now(self) -> float:
        return time.monotonic()

    def sleep(self, seconds: float) -> None:
        time.sleep(seconds)


@dataclass(frozen=True)
class Decision:
    """Outcome of a non-blocking call.

    `retry_after` is the number of seconds to wait before the same call would
    be admitted; it is 0.0 when the call was allowed.
    """

    allowed: bool
    retry_after: float


class TokenBucket:
    """Rate-limit state for one key. Not thread-safe; the limiter serialises access."""

    def __init__(self, rate: float, capacity: float, now: float) -> None:
        self._rate = rate
        self._capacity = capacity
        self._tokens = capacity
        self._last = now

    def take(self, now: float) -> Decision:
        """Refill from elapsed time, then consume one token if one is available."""
        self._tokens = min(
            self._capacity, self._tokens + (now - self._last) * self._rate
        )
        self._last = now
        if self._tokens >= 1:
            self._tokens -= 1
            return Decision(True, 0.0)
        return Decision(False, (1 - self._tokens) / self._rate)


class RateLimiter:
    """Per-key token-bucket rate limiter.

    `rate` is tokens added per second and `capacity` is the burst size; both
    apply to every key. Each call consumes one token.
    """

    def __init__(
        self, rate: float, capacity: float, *, clock: Clock | None = None
    ) -> None:
        self._rate = rate
        self._capacity = capacity
        self._clock = clock if clock is not None else SystemClock()
        self._buckets: dict[str, TokenBucket] = {}
        self._lock = threading.Lock()  # one lock for all state (ADR-RL-004)

    def try_acquire(self, key: str) -> Decision:
        """Admit the call if a token is available, otherwise refuse it."""
        with self._lock:
            now = self._clock.now()  # read under the lock so time never runs backwards
            bucket = self._buckets.get(key)
            if bucket is None:
                bucket = self._buckets[key] = TokenBucket(
                    self._rate, self._capacity, now
                )
            return bucket.take(now)

    def acquire(self, key: str) -> None:
        """Block until the call is admitted. Sleeps without holding the lock."""
        while True:
            decision = self.try_acquire(key)
            if decision.allowed:
                return
            self._clock.sleep(decision.retry_after)
