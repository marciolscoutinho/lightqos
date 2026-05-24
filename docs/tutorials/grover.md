# 🎓 Tutorial — Grover's Algorithm with LightQOS

> Prerequisite: [`getting_started.md`](getting_started.md)

---

## What Is Grover's Algorithm?

Grover's algorithm (1996) is a quantum algorithm for **unstructured search**. Given a function f: {0,...,N-1} → {0,1} with exactly one solution x* such that f(x*)=1, it finds x* in:

- **Classical**: O(N) evaluations
- **Grover**: O(√N) evaluations — **quadratic speedup**

For N=1,000,000 items, the classical approach uses ~500,000 steps, while Grover uses ~1,000 steps.

---

## Implementation with LightQOS

### Step 1 — Install and import

```python
from lightqos import QuantumCircuit, TemporalContract
import math
```

### Step 2 — Build the Oracle

The oracle marks the element x* by applying a -1 phase:

```python
def build_grover_oracle(n_qubits: int, target: int) -> QuantumCircuit:
    """
    Phase oracle that marks |target⟩:
    |x⟩ → -|x⟩  if x == target
    |x⟩ →  |x⟩  if x != target
    """
    oracle = QuantumCircuit(n_qubits)

    # Convert target to binary and apply X to the 0 bits
    target_bits = format(target, f'0{n_qubits}b')

    for i, bit in enumerate(reversed(target_bits)):
        if bit == '0':
            oracle.x(i)  # Flip so that the target becomes all 1s

    # Multi-controlled Z, equivalent to generalized CZ
    oracle.mcz(list(range(n_qubits)))

    # Undo the X gates
    for i, bit in enumerate(reversed(target_bits)):
        if bit == '0':
            oracle.x(i)

    return oracle
```

### Step 3 — Grover Diffuser

The diffuser applies the reflection 2|s⟩⟨s| - I:

```python
def build_grover_diffuser(n_qubits: int) -> QuantumCircuit:
    """Diffuser: inversion about the mean."""
    diffuser = QuantumCircuit(n_qubits)

    # H on all qubits
    for q in range(n_qubits):
        diffuser.h(q)

    # X on all qubits
    for q in range(n_qubits):
        diffuser.x(q)

    # Multi-controlled Z
    diffuser.mcz(list(range(n_qubits)))

    # Undo X on all qubits
    for q in range(n_qubits):
        diffuser.x(q)

    # Undo H on all qubits
    for q in range(n_qubits):
        diffuser.h(q)

    return diffuser
```

### Step 4 — Complete Grover Circuit

```python
def grover_search(n_qubits: int, target: int, shots: int = 1024) -> dict:
    """
    Runs Grover's algorithm to find 'target' in a search space of 2^n_qubits.

    Args:
        n_qubits: Number of search qubits
        target:   Element to find (0 ≤ target < 2^n_qubits)
        shots:    Number of executions

    Returns:
        Counts dictionary {'0000': 5, '1011': 1019, ...}
    """
    N = 2 ** n_qubits
    n_iterations = math.floor(math.pi / 4 * math.sqrt(N))  # Optimal: π√N/4 iterations

    print(f"Grover: n={n_qubits} qubits, N={N} states, target={target}, iterations={n_iterations}")

    # Build components
    oracle = build_grover_oracle(n_qubits, target)
    diffuser = build_grover_diffuser(n_qubits)

    # Main circuit
    circuit = QuantumCircuit(n_qubits)

    # Initialization: uniform superposition
    for q in range(n_qubits):
        circuit.h(q)

    # Grover iterations
    for iteration in range(n_iterations):
        circuit.compose(oracle)    # Apply oracle
        circuit.compose(diffuser)  # Apply diffuser

    # Measure
    circuit.measure(list(range(n_qubits)))

    # Execute with temporal contract
    contract = TemporalContract(
        operation=f"grover_{n_qubits}q",
        deadline_ms=5000.0,
        priority=8
    )

    result = circuit.execute(backend="simulator", shots=shots, contract=contract)
    return result.counts
```

### Step 5 — Run and verify

```python
# Search over 4 qubits (16 states)
n_qubits = 4
target = 11  # '1011' in binary

counts = grover_search(n_qubits, target, shots=1024)

# Show sorted results
sorted_counts = sorted(counts.items(), key=lambda x: x[1], reverse=True)
print("\nResults (top 5):")
for bitstring, count in sorted_counts[:5]:
    value = int(bitstring, 2)
    marker = " ← TARGET" if value == target else ""
    print(f"  |{bitstring}⟩ ({value:2d})  —  {count:4d}  ({100*count/1024:.1f}%){marker}")

# Verify success
target_bitstring = format(target, f'0{n_qubits}b')
success_prob = counts.get(target_bitstring, 0) / 1024
print(f"\nProbability of finding target: {success_prob:.1%}")
assert success_prob > 0.9, f"Algorithm failed! P={success_prob:.1%}"
print("✅ Grover algorithm successful!")
```

**Expected output:**

```
Grover: n=4 qubits, N=16 states, target=11, iterations=3

Results (top 5):
  |1011⟩ (11)  —  1019  (99.5%) ← TARGET
  |0010⟩ ( 2)  —     2  ( 0.2%)
  |1100⟩ (12)  —     1  ( 0.1%)
  |0101⟩ ( 5)  —     1  ( 0.1%)
  |1110⟩ (14)  —     1  ( 0.1%)

Probability of finding target: 99.5%
✅ Grover algorithm successful!
```

---

## Scalability Analysis

```python
import math

print(f"{'n':>4} {'N':>8} {'Iterations':>10} {'P(success)':>12}")
print("-" * 40)

for n in [2, 3, 4, 5, 6, 8, 10, 15, 20]:
    N = 2 ** n
    iters = math.floor(math.pi / 4 * math.sqrt(N))
    # Theoretical probability
    theta = math.asin(1.0 / math.sqrt(N))
    prob = math.sin((2 * iters + 1) * theta) ** 2
    print(f"{n:>4} {N:>8} {iters:>10} {prob:>11.1%}")
```

---

## Version with Shadow Tomography

To verify the quality of the intermediate state:

```python
# Grover with Shadow Tomography verification
circuit_with_shadow = QuantumCircuit(n_qubits, shadow_tomography=True)

for q in range(n_qubits):
    circuit_with_shadow.h(q)

for i in range(n_iterations):
    circuit_with_shadow.compose(oracle)
    circuit_with_shadow.compose(diffuser)

    # Verify state after each iteration
    shadow_result = circuit_with_shadow.shadow_checkpoint(
        n_shadows=200,
        observables=["Z0", "Z1", "Z2", "Z3"]
    )
    print(f"Iteration {i+1}: estimated fidelity = {shadow_result.mean_fidelity:.3f}")

circuit_with_shadow.measure(list(range(n_qubits)))
result = circuit_with_shadow.execute(backend="simulator", shots=1024)
```

---

## References

- Grover, L. K. (1996). *A fast quantum mechanical algorithm for database search*. Proceedings of the 28th STOC, 212-219.
- Nielsen, M. A., & Chuang, I. L. (2010). *Quantum Computation and Quantum Information*. Cambridge University Press. Ch. 6.
