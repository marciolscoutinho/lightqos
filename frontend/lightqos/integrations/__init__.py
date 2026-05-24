# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# __init__.py — Framework Integrations — Qiskit, Cirq, PennyLane adapters
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 26-04-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
LightQOS Frontend Integrations

Adapters for external quantum frameworks:
- Qiskit (IBM Quantum)
- Cirq (Google)
- PennyLane (Xanadu)
"""

from .qiskit_adapter import qiskit_to_lightqos, lightqos_to_qiskit
from .cirq_adapter import cirq_to_lightqos, lightqos_to_cirq
from .pennylane_adapter import pennylane_to_lightqos

__all__ = [
    "qiskit_to_lightqos",
    "lightqos_to_qiskit",
    "cirq_to_lightqos",
    "lightqos_to_cirq",
    "pennylane_to_lightqos",
]
