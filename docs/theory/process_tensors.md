# 🔬 Process Tensor Framework — Theoretical Foundation

> Based on: Milz, S., & Modi, K. (2021). *Quantum stochastic processes and quantum non-Markovian phenomena*. PRX Quantum, 2(3), 030201.

---

## 1. Motivation — Beyond Markovian Channels

### Classical CPTP quantum channel

The standard model of quantum noise uses **CPTP channels**: Completely Positive, Trace-Preserving maps:

```
ρ_out = Σᵢ Kᵢ ρ_in Kᵢ†     (Kraus representation)
```

This assumes **zero memory** — the environment “forgets” everything between consecutive operations.

### The problem: real hardware has memory

In real hardware, such as superconducting systems and trapped ions:

- **Crosstalk**: a gate on qubit A affects qubit B in the following cycles
- **Frequency drift**: the transition frequency changes slowly over time
- **1/f noise**: long-range temporal correlations
- **Environmental feedback**: the quantum bath retains information

To model this correctly, we need **Process Tensors**.

---

## 2. Quantum Stochastic Process

### Formal definition

A **quantum process** Υ_{T:0} of order T is a map that, given the full history of interventions {A_t}_{t=0}^{T-1}, produces the final state:

```
ρ_T = Υ_{T:0} [A_{T-1} ⊗ A_{T-2} ⊗ ... ⊗ A_0]
```

where each Aₜ is a quantum operation, or CPTP instrument.

### Tensor representation

The process Υ_{T:0} can be represented as a **process tensor**: a density matrix in an extended Hilbert space:

```
T_Υ ∈ L(H_E ⊗ H_{o,T-1} ⊗ H_{i,T-1} ⊗ ... ⊗ H_{o,0} ⊗ H_{i,0})
```

where H_{i,t} and H_{o,t} are the input and output spaces at each time step t.

---

## 3. Non-Markovianity

### Measure of non-Markovianity

A process is **Markovian** if and only if:

```
T_Υ = ρ_0 ⊗ ρ_1 ⊗ ... ⊗ ρ_{T-1}    (tensor product)
```

The **non-Markovianity measure** N is:

```
N(Υ) = min_{Markov M} D(Υ, M)
```

where D is the trace distance. N=0 means Markovian; N>0 means non-Markovian.

### Implementation in LightQOS

```rust
// kernel/src/tlm/process_tensor.rs
pub struct ProcessTensor {
    order: usize,           // Number of steps T
    tensor_data: Vec<C64>,  // Tensor data (matrix representation)
    hilbert_dim: usize,     // Hilbert-space dimension per step
}

impl ProcessTensor {
    pub fn non_markovianity_measure(&self) -> f64 {
        let markov_approx = self.best_markov_approximation();
        self.trace_distance(&markov_approx)
    }
    
    pub fn add_step(&mut self, channel: &KrausChannel) {
        // Adds one step to the process
        self.tensor_data = self.extend_tensor(channel);
        self.order += 1;
    }
}
```

---

## 4. Quantum Combs

### Definition

An **n-order quantum comb** is a sequence of superoperators with fixed causal structure:

```
C^(n): (CPTP₁, CPTP₂, ..., CPTPₙ) → ρ_final
```

Visually, it resembles a “comb”:

```
       A₁       A₂       A₃
       ↓        ↓        ↓
───[C₁]──────[C₂]──────[C₃]──→ ρ_out
```

### Advantage over Process Tensor

Combs have an **explicit causal structure**: we know that C₁ occurs before C₂, and so on. This reduces the number of free parameters from O(d^{4T}) to O(d^{2T}).

### Implementation

```rust
// process_tensor/src/quantum_comb.rs
pub struct QuantumComb {
    slots: Vec<CombSlot>,   // Each slot = one allowed intervention
    causal_order: Vec<usize>,
}

impl QuantumComb {
    pub fn apply(&self, interventions: &[CPTPMap]) -> QuantumState {
        assert_eq!(interventions.len(), self.slots.len());
        
        let mut state = self.initial_state.clone();
        for (slot, intervention) in self.slots.iter().zip(interventions) {
            state = slot.propagate(state);
            state = intervention.apply(state);
        }
        state
    }
    
    pub fn link_product(&self, other: &QuantumComb) -> QuantumComb {
        // Link product: causally composes two combs
        todo!()
    }
}
```

---

## 5. Memory Kernel

The **memory kernel** K(t, t') describes how the state at t' influences the dynamics at t:

### Generalized GKSL equation (non-Markovian)

```
dρ/dt = ∫₀ᵗ K(t, t') ρ(t') dt'    (Nakajima-Zwanzig)
```

For the Markovian case: K(t, t') = L·δ(t-t') (Lindblad).

### Implementation

```rust
// process_tensor/src/memory_kernel.rs
pub struct MemoryKernel {
    kernel_matrix: Vec<Vec<f64>>,  // K(t, t')
    time_steps: Vec<f64>,
    markov_rank: usize,            // 1 = Markovian
}

impl MemoryKernel {
    /// Computes the environment correlation time τ_E
    pub fn correlation_time(&self) -> f64 {
        let k_norms: Vec<f64> = self.kernel_matrix.iter()
            .map(|row| row.iter().map(|&x| x * x).sum::<f64>().sqrt())
            .collect();
        
        // τ_E ≈ ∫₀^∞ |K(t)| dt / |K(0)|
        let integral: f64 = k_norms.windows(2)
            .zip(self.time_steps.windows(2))
            .map(|(k, t)| (k[0] + k[1]) / 2.0 * (t[1] - t[0]))
            .sum();
        
        integral / k_norms[0].max(1e-15)
    }
    
    /// Applies the kernel to state evolution
    pub fn evolve(&self, rho_history: &[DensityMatrix], dt: f64) -> DensityMatrix {
        // Discrete convolution: dρ/dt = Σ_{t'} K(t, t') ρ(t') Δt
        let n = rho_history.len();
        let dim = rho_history[0].len();
        let mut drho = vec![vec![C64::default(); dim]; dim];
        
        for (t_prime, rho) in rho_history.iter().enumerate() {
            let k = self.kernel_matrix[n - 1][t_prime];
            for i in 0..dim {
                for j in 0..dim {
                    drho[i][j] += k * rho[i][j] * dt;
                }
            }
        }
        drho
    }
}
```

---

## 6. Use in LightQOS

### TLM — Temporal management with Process Tensor

TLM uses the Process Tensor to **model the operation history** and compensate for memory effects:

```python
from lightqos.tlm import ProcessTensorManager

ptm = ProcessTensorManager(order=5)  # 5-step memory

# Register each operation
ptm.record(channel="H_gate", qubit=0, timestamp_ns=0)
ptm.record(channel="CNOT", qubits=[0,1], timestamp_ns=50)

# Compute non-Markovianity compensation
compensation = ptm.compute_correction()

# Apply it to the next gate
corrected_circuit = ptm.apply_correction(circuit, compensation)
```

### Non-Markovianity benchmarking

```python
from lightqos.diagnostics import NonMarkovianityBenchmark

bench = NonMarkovianityBenchmark(driver=ibm_driver)
results = await bench.run(n_steps=10, n_shots=1024)

print(f"N = {results.non_markovianity:.4f}")  # 0 = Markovian
print(f"τ_E = {results.correlation_time_ns:.1f} ns")
print(f"Significant memory up to: {results.memory_depth} steps")
```

---

## 7. References

1. Milz, S., & Modi, K. (2021). *Quantum stochastic processes and quantum non-Markovian phenomena*. PRX Quantum, 2(3), 030201.
2. Pollock, F. A., Rodríguez-Rosario, C., Frauenheim, T., Paternostro, M., & Modi, K. (2018). *Non-Markovian quantum processes: Complete framework and efficient characterization*. Physical Review A, 97(1), 012127.
3. Chiribella, G., D'Ariano, G. M., & Perinotti, P. (2009). *Theoretical framework for quantum networks*. Physical Review A, 80(2), 022339.
4. Jorgensen, M. R., & Pollock, F. A. (2019). *Exploiting the causal tensor network structure of quantum processes to efficiently simulate non-Markovian path integrals*. Physical Review Letters, 123(24), 240602.
