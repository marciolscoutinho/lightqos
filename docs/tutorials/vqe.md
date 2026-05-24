# 🎓 Tutorial — VQE (Variational Quantum Eigensolver)

> Prerequisite: [`getting_started.md`](getting_started.md)  
> Requires: `pip install scipy numpy`

---

## What Is VQE?

The **VQE** (Peruzzo et al., 2014) is a hybrid quantum-classical algorithm used to find the **minimum-energy state** of a Hamiltonian:

```
E₀ = min_θ ⟨ψ(θ)|H|ψ(θ)⟩
```

- The quantum circuit prepares `|ψ(θ)⟩`, a parameterized ansatz
- The classical optimizer adjusts θ to minimize the energy
- The process iterates until convergence

Applications include quantum chemistry, combinatorial optimization and materials simulation.

---

## Complete Implementation

```python
from lightqos import QuantumCircuit
import numpy as np
from scipy.optimize import minimize

# =====================================================
# 1. Define Hamiltonian: simplified H₂ example
# =====================================================
# H = a*ZZ + b*XX + c*ZI + d*IZ
# Coefficients for H₂ at equilibrium distance (STO-3G)
H_COEFFS = {
    "ZZ": -0.1809,
    "XX":  0.1809,
    "ZI": -0.4546,
    "IZ": -0.4546,
    "II":  0.7238   # nuclear constant
}

def pauli_expectation(circuit: QuantumCircuit, pauli_string: str, shots: int = 2048) -> float:
    """Computes ⟨ψ|P|ψ⟩ for a Pauli string, for example 'ZZ', 'XX' or 'ZI'."""
    n = len(pauli_string)
    meas_circuit = circuit.copy()

    # Basis change to measure non-Z bases
    for i, p in enumerate(pauli_string):
        if p == 'X':
            meas_circuit.h(i)          # H|ψ⟩ → measuring Z is equivalent to measuring X
        elif p == 'Y':
            meas_circuit.sdg(i)        # S†H|ψ⟩ → measuring Z is equivalent to measuring Y
            meas_circuit.h(i)
        # Z: no basis change required

    meas_circuit.measure(list(range(n)))
    result = meas_circuit.execute(backend="simulator", shots=shots)

    # Compute expected value: +1 for even parity, -1 for odd parity
    expval = 0.0
    for bitstring, count in result.counts.items():
        # Count active qubits, excluding 'I'
        parity = 0
        for i, (bit, p) in enumerate(zip(bitstring, pauli_string)):
            if p != 'I':
                parity ^= int(bit)
        sign = 1 - 2 * parity  # +1 or -1
        expval += sign * count / shots

    return expval

def hamiltonian_expectation(circuit: QuantumCircuit, shots: int = 2048) -> float:
    """Computes ⟨ψ|H|ψ⟩ = Σ cᵢ ⟨ψ|Pᵢ|ψ⟩."""
    total = H_COEFFS.get("II", 0.0)  # constant
    for pauli, coeff in H_COEFFS.items():
        if pauli != "II":
            expval = pauli_expectation(circuit, pauli, shots)
            total += coeff * expval
    return total

# =====================================================
# 2. Parameterized ansatz: RY-CNOT layer
# =====================================================
def build_ansatz(theta: np.ndarray, n_qubits: int = 2, depth: int = 1) -> QuantumCircuit:
    """
    Hardware Efficient Ansatz (HEA):
    alternating layers of RY rotations and CNOTs.

    Parameters per layer: n_qubits, one RY per qubit
    Total: n_qubits * depth parameters
    """
    assert len(theta) == n_qubits * depth, "Incorrect number of parameters"

    circuit = QuantumCircuit(n_qubits)

    # Initial superposition
    for q in range(n_qubits):
        circuit.h(q)

    # HEA layers
    param_idx = 0
    for d in range(depth):
        # Parameterized rotations
        for q in range(n_qubits):
            circuit.ry(theta[param_idx], q)
            param_idx += 1

        # Entanglement: CNOT ladder
        for q in range(n_qubits - 1):
            circuit.cnot(q, q + 1)
        if n_qubits > 2:
            circuit.cnot(n_qubits - 1, 0)  # Circular closure

    return circuit

# =====================================================
# 3. Classical optimization
# =====================================================
n_qubits = 2
depth = 2
n_params = n_qubits * depth

# Convergence history
energy_history = []

def cost_function(theta: np.ndarray) -> float:
    """Cost function = Hamiltonian energy."""
    ansatz = build_ansatz(theta, n_qubits, depth)
    energy = hamiltonian_expectation(ansatz, shots=1024)
    energy_history.append(energy)
    return energy

# Random initial point
theta_init = np.random.uniform(-np.pi, np.pi, n_params)
print(f"Initial energy: {cost_function(theta_init):.6f} Ha")

# Optimization with COBYLA, which does not require gradients
result_opt = minimize(
    cost_function,
    theta_init,
    method="COBYLA",
    options={"maxiter": 500, "rhobeg": 0.1}
)

print(f"\n{'='*40}")
print(f"VQE converged after {len(energy_history)} evaluations")
print(f"Minimum energy found:      {result_opt.fun:.6f} Ha")
print(f"Exact energy (FCI):        -1.137270 Ha")
print(f"Error:                     {abs(result_opt.fun - (-1.137270)):.6f} Ha")
print(f"{'='*40}")

# =====================================================
# 4. Visualize convergence
# =====================================================
print("\nEnergy convergence:")
for i, e in enumerate(energy_history[::20]):  # Every 20 iterations
    bar = "█" * max(0, int((e + 2.0) * 10))
    print(f"  Iter {i*20:3d}: {e:.4f} Ha  {bar}")
```

---

## Quantum Gradients: Parameter Shift Rule

For gradient-based optimizers:

```python
def parameter_shift_gradient(theta: np.ndarray, param_idx: int) -> float:
    """
    Gradient via Parameter Shift Rule:
    ∂E/∂θᵢ = [E(θᵢ + π/2) - E(θᵢ - π/2)] / 2
    """
    shift = np.pi / 2

    theta_plus = theta.copy()
    theta_plus[param_idx] += shift

    theta_minus = theta.copy()
    theta_minus[param_idx] -= shift

    E_plus  = cost_function(theta_plus)
    E_minus = cost_function(theta_minus)

    return (E_plus - E_minus) / 2
```

---

## References

- Peruzzo, A., et al. (2014). *A variational eigenvalue solver on a photonic chip*. Nature Communications, 5, 4213.
- Tilly, J., et al. (2022). *The variational quantum eigensolver: A review of methods and best practices*. Physics Reports, 986, 1-128.
