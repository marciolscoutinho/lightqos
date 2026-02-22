"""
Example: Bell State with a Temporal Contract
"""

from lightqos import QuantumCircuit, TemporalContract

# Create circuit
circuit = QuantumCircuit(2, name="bell_state")

# Define temporal contracts
fast_sla = TemporalContract(
    max_latency_ns=50,   # 50 nanoseconds max
    deadline_phase=0.1,  # Relative phase
    rollback_on_violation=True
)

# Build the Bell state with SLAs
circuit.h(0, sla=fast_sla)
circuit.cx(0, 1, sla=fast_sla)

# Holographic measurement (HIO)
circuit.measure([0, 1], holographic=True)

# Run on IonQ Forte
results = circuit.execute(backend='ionq_forte', shots=1024)

print("Results:", results)
print("HIO Data:", results.get('hio_data'))
