# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# __init__.py — LightQOS Python SDK — package entry point and public API
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 18-06-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
LightQOS — Light Quantum Operating System
==========================================

High-level Python SDK for LightQOS.

Quick example:
    >>> from lightqos import QuantumCircuit, TemporalContract
    >>> qc = QuantumCircuit(2)
    >>> qc.h(0)
    >>> qc.cnot(0, 1)
    >>> qc.measure([0, 1])
    >>> result = qc.execute(backend="simulator", shots=1024)
    >>> print(result.counts)
    {'00': 512, '11': 512}

GitHub: https://github.com/marciolscoutinho/lightqos
"""

__version__ = "0.2.0"
__author__ = "Márcio Coutinho"

from .circuit import QuantumCircuit
from .contracts import TemporalContract, QoSContract

# Import integrations optionally (Qiskit/Cirq may not be installed)
try:
    from .integrations import (
        qiskit_to_lightqos as qiskit_to_lightqos,
        lightqos_to_qiskit as lightqos_to_qiskit,
    )
except ImportError:
    pass

try:
    from .integrations import (
        cirq_to_lightqos as cirq_to_lightqos,
        lightqos_to_cirq as lightqos_to_cirq,
    )
except ImportError:
    pass

__all__ = [
    "__version__",
    "QuantumCircuit",
    "TemporalContract",
    "QoSContract",
]
