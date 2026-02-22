"""
Example: Integration with Qiskit
"""

from qiskit import QuantumCircuit as QiskitCircuit
from lightqos.integrations import qiskit_to_lightqos, lightqos_to_qiskit
from lightqos import TemporalContract

# Create a circuit in Qiskit
qiskit_circ = QiskitCircuit(3)
qiskit_circ.h(0)
qiskit_circ.cx(0, 1)
qiskit_circ.cx(1, 2)
qiskit_circ.measure_all()

print("Original Qiskit circuit:")
print(qiskit_circ)

# Convert to LightQOS
lightqos_circ = qiskit_to_lightqos(qiskit_circ)

print("\nConverted to LightQOS:")
print(lightqos_circ.to_qasm3())

# Execute with LightQOS features
lightqos_circ.add_temporal_contract(TemporalContract.strict())
results = lightqos_circ.execute(backend="simulator", shots=1024)

print("\nResults:", results)
