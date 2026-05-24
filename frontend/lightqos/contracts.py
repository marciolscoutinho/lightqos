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
