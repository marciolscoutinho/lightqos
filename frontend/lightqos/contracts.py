# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# contracts.py — Temporal Contracts — SLA and QoS contract management
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 08-08-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Temporal and Quality of Service Contracts
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class TemporalContract:
    """
    Temporal contract for quantum operations (SLA)

    Attributes:
        max_latency_ns: Maximum allowed latency, in nanoseconds
        deadline_phase: Phase deadline, from 0.0 to 1.0
        rollback_on_violation: Whether rollback should be performed on violation
        max_retries: Maximum number of retries
    """

    max_latency_ns: int = 1000
    deadline_phase: float = 0.1
    rollback_on_violation: bool = False
    max_retries: int = 1

    def validate(self) -> bool:
        """Validates the contract"""
        if self.max_latency_ns <= 0:
            raise ValueError("max_latency_ns must be positive")

        if not (0.0 < self.deadline_phase <= 1.0):
            raise ValueError("deadline_phase must be in (0, 1]")

        if self.max_retries < 1:
            raise ValueError("max_retries must be >= 1")

        return True

    @classmethod
    def strict(cls) -> "TemporalContract":
        """Creates a strict contract, high performance"""
        return cls(
            max_latency_ns=100,
            deadline_phase=0.05,
            rollback_on_violation=True,
            max_retries=3,
        )

    @classmethod
    def permissive(cls) -> "TemporalContract":
        """Creates a permissive contract"""
        return cls(
            max_latency_ns=10000,
            deadline_phase=0.5,
            rollback_on_violation=False,
            max_retries=1,
        )


@dataclass
class QoSContract:
    """
    Quality of Service contract

    Attributes:
        min_fidelity: Minimum fidelity, from 0.0 to 1.0
        min_coherence_time_ns: Minimum coherence time, in nanoseconds
        target_platform: Specific target platform
        error_mitigation: Whether error mitigation should be used
    """

    min_fidelity: float = 0.99
    min_coherence_time_ns: int = 100000  # 100 μs
    target_platform: Optional[str] = None
    error_mitigation: bool = False

    def validate(self) -> bool:
        """Validates the contract"""
        if not (0.0 <= self.min_fidelity <= 1.0):
            raise ValueError("min_fidelity must be in [0, 1]")

        if self.min_coherence_time_ns <= 0:
            raise ValueError("min_coherence_time_ns must be positive")

        return True

    @classmethod
    def high_quality(cls) -> "QoSContract":
        """High-quality contract"""
        return cls(
            min_fidelity=0.999,
            min_coherence_time_ns=1000000,  # 1 ms
            error_mitigation=True,
        )

    @classmethod
    def low_quality(cls) -> "QoSContract":
        """Low-quality contract, NISQ"""
        return cls(
            min_fidelity=0.9,
            min_coherence_time_ns=10000,  # 10 μs
            error_mitigation=False,
        )


# --- LightQOS test compatibility TemporalContract ---
import time as _lq_time  # noqa: E402
import uuid as _lq_uuid  # noqa: E402


class TemporalContract:  # noqa: F811
    """Compatibility temporal contract used by the Python SDK tests.

    Supports both the test-facing API:
        TemporalContract(operation="CNOT", deadline_ms=100.0, priority=5)

    and the older LightQOS-style parameters:
        TemporalContract(max_latency_ns=..., deadline_phase=...)
    """

    def __init__(
        self,
        operation=None,
        deadline_ms=None,
        priority=5,
        max_latency_ns=None,
        deadline_phase=None,
        rollback_on_violation=False,
        max_retries=1,
    ):
        if deadline_ms is None and isinstance(operation, (int, float)):
            deadline_ms = float(operation)
            operation = "UNKNOWN"

        self.operation = str(operation) if operation is not None else "UNKNOWN"
        self.deadline_ms = float(deadline_ms) if deadline_ms is not None else 1000.0
        self.priority = max(1, min(10, int(priority)))
        self.id = str(_lq_uuid.uuid4())
        self.created_at = _lq_time.monotonic()
        self.fulfilled = False

        self.max_latency_ns = max_latency_ns
        self.deadline_phase = deadline_phase
        self.rollback_on_violation = rollback_on_violation
        self.max_retries = max_retries

    def time_remaining_ms(self):
        elapsed_ms = (_lq_time.monotonic() - self.created_at) * 1000.0
        return max(0.0, self.deadline_ms - elapsed_ms)

    def is_expired(self):
        return self.time_remaining_ms() <= 0.0

    def __repr__(self):
        return (
            "TemporalContract("
            f"operation={self.operation!r}, "
            f"deadline_ms={self.deadline_ms!r}, "
            f"priority={self.priority!r}, "
            f"fulfilled={self.fulfilled!r}"
            ")"
        )


# --- End LightQOS test compatibility TemporalContract ---
