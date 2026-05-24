# 🎓 Tutorial — Quantum Fourier Transform (QFT)

> Prerequisite: [`getting_started.md`](getting_started.md)

---

## What Is the QFT?

The **Quantum Fourier Transform** is the quantum version of the Discrete Fourier Transform (DFT). For n qubits, it maps:

```
QFT|j⟩ = (1/√N) Σₖ e^(2πijk/N) |k⟩
```

- **Classical DFT**: O(N log N) operations (FFT)
- **Quantum QFT**: O(n²) gates — **exponentially more efficient**

It is the core of algorithms such as Shor's algorithm for factorization and Quantum Phase Estimation.

---

## Implementation

```python
from lightqos import QuantumCircuit
import math

def qft_circuit(n_qubits: int) -> QuantumCircuit:
    """Quantum Fourier Transform on n qubits."""
    circuit = QuantumCircuit(n_qubits)

    for j in range(n_qubits):
        # Hadamard on qubit j
        circuit.h(j)

        # Controlled-R_k gates
        for k in range(j + 1, n_qubits):
            angle = 2 * math.pi / (2 ** (k - j + 1))
            circuit.cp(angle, k, j)  # Controlled phase

    # SWAP to reverse order
    for i in range(n_qubits // 2):
        circuit.swap(i, n_qubits - i - 1)

    return circuit

def iqft_circuit(n_qubits: int) -> QuantumCircuit:
    """Inverse QFT."""
    qft = qft_circuit(n_qubits)
    return qft.inverse()
```

### Run

```python
n = 4
circuit = QuantumCircuit(n)

# Input state: |5⟩ = |0101⟩
circuit.x(0)  # bit 0
circuit.x(2)  # bit 2

# Apply QFT
circuit.compose(qft_circuit(n))
circuit.measure(list(range(n)))

result = circuit.execute(backend="simulator", shots=2048)
print("QFT of |5⟩:")
for state, count in sorted(result.counts.items(), key=lambda x: -x[1])[:5]:
    print(f"  |{state}⟩: {count} ({100*count/2048:.1f}%)")
```

---

## Application: Phase Estimation

The QFT is essential for estimating eigenvalues, or phases, of unitary operators:

```python
def quantum_phase_estimation(unitary_circuit, n_ancilla: int = 4) -> QuantumCircuit:
    """
    Estimates the phase φ such that U|ψ⟩ = e^(2πiφ)|ψ⟩

    Args:
        unitary_circuit: Circuit implementing U
        n_ancilla:       Precision bits
    """
    n_eigen = unitary_circuit.n_qubits
    total_qubits = n_ancilla + n_eigen

    circuit = QuantumCircuit(total_qubits, n_ancilla)

    # Initialize ancilla in superposition
    for q in range(n_ancilla):
        circuit.h(q)

    # Apply U^(2^k) controlled by each ancilla
    for k in range(n_ancilla):
        reps = 2 ** k
        controlled_U = unitary_circuit.controlled(control_qubit=k)
        for _ in range(reps):
            circuit.compose(controlled_U)

    # Inverse QFT on ancilla
    circuit.compose(iqft_circuit(n_ancilla), qubits=list(range(n_ancilla)))

    # Measure ancilla
    circuit.measure(list(range(n_ancilla)), list(range(n_ancilla)))

    return circuit

# Example: estimate the phase of the T gate (φ = 1/8)
from lightqos.gates import TGate

t_gate_circuit = QuantumCircuit(1)
t_gate_circuit.t(0)

qpe = quantum_phase_estimation(t_gate_circuit, n_ancilla=4)
result = qpe.execute(backend="simulator", shots=2048)

# Convert most frequent result into phase
most_common = max(result.counts, key=result.counts.get)
measured_phase = int(most_common, 2) / (2 ** 4)
print(f"Measured phase: {measured_phase:.3f}")
print(f"True phase:     {1/8:.3f}")
```

---

## References

- Nielsen, M. A., & Chuang, I. L. (2010). *Quantum Computation and Quantum Information*. Ch. 5.
- Coppersmith, D. (1994). *An approximate Fourier transform useful in quantum factoring*. IBM Research Report RC19642.
