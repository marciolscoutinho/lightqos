# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# basic_circuit.py — Basic Circuit Example — simple single and multi-qubit circuits
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 15-02-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Basic example: Simple circuit
"""

from lightqos import QuantumCircuit

# Creates a 2-qubit circuit
circuit = QuantumCircuit(2, name="basic_example")

# Applies gates
circuit.h(0)
circuit.cx(0, 1)
circuit.measure([0, 1])

# Prints QASM
print(circuit.to_qasm3())

# Executes, simulator
results = circuit.execute(backend="simulator", shots=1024)
print("Results:", results)
