# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# high_dimensional_ghz.py — High-Dimensional GHZ — multi-qubit entanglement example
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 24-05-2026
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Advanced example: High-dimensional GHZ state
Inspired by the 37-dimensional photon experiment
"""

from lightqos import QuantumCircuit, TemporalContract

# Creates a 37-qubit circuit, simulating 37 dimensions
circuit = QuantumCircuit(37, name="ghz_37d")

# Strict temporal contract
strict_sla = TemporalContract.strict()

# Prepairs the GHZ state |000...0⟩ + |111...1⟩
circuit.h(0, sla=strict_sla)

for i in range(1, 37):
    circuit.cx(0, i, sla=strict_sla)

    # Holographic measurement
circuit.measure(list(range(37)), holographic=True)

# Executes
print("Executing 37-dimensional GHZ state...")
results = circuit.execute(backend="simulator", shots=100)

print("HIO Results:")
print(f"  Confidence: {results.get('hio_data', {}).get('confidence', 'N/A')}")
print(f"  Fidelity: {results.get('hio_data', {}).get('fidelity', 'N/A')}")
