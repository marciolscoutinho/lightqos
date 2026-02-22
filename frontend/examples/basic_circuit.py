"""
Basic example: Simple circuit
"""

from lightqos import QuantumCircuit

# Create a 2-qubit circuit
circuit = QuantumCircuit(2, name="basic_example")

# Apply gates
circuit.h(0)
circuit.cx(0, 1)
circuit.measure([0, 1])

# Print QASM
print(circuit.to_qasm3())

# Execute (simulator)
results = circuit.execute(backend='simulator', shots=1024)
print("Results:", results)
