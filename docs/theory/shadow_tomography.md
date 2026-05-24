# 👁️ Shadow Tomography — Theoretical Foundation

> Based on: Aaronson, S. (2019). *Shadow tomography of quantum states*. SIAM Journal on Computing.

---

## 1. The Quantum Tomography Problem

Full quantum state tomography requires **exponentially many measurements**:

- For n qubits, the state ρ is a 2ⁿ × 2ⁿ matrix of complex numbers
- To fully reconstruct ρ, we need O(4ⁿ) measurements
- For n=50: 10³⁰ measurements — completely infeasible

### What do we actually want?

In practice, we do not need the full ρ. We need **M specific observables**:

```
E₁(ρ) = Tr(O₁ρ), E₂(ρ) = Tr(O₂ρ), ..., Eₘ(ρ) = Tr(Oₘρ)
```

---

## 2. Aaronson Shadow Tomography (2019)

### Main theorem

> Given an unknown n-qubit state ρ and M observables O₁, ..., Oₘ with ‖Oᵢ‖ ≤ 1, it is possible to estimate all Tr(Oᵢρ) values with error ε using only:

```
T = O(log²M · log n · ε⁻⁴) measurements (original algorithm)
T = O(log M · ε⁻²) measurements (improved classical shadows version)
```

### Complexity comparison

| Method | Measurements | n=10 | n=50 |
|--------|--------------|------|------|
| Full tomography | O(4ⁿ) | 10⁶ | 10³⁰ |
| Shadow Tomography | O(log M · ε⁻²) | 46k | 46k |

For M=1000 observables and ε=0.01: **constant independent of n**!

---

## 3. Classical Shadows (Huang et al., 2020)

Huang, Kueng & Preskill (2020) simplified the protocol:

### Three-step protocol

**Step 1 — Apply random rotation**

```
Choose random U from the Clifford group
Apply U to the state: ρ → UρU†
```

**Step 2 — Measure in the computational basis**

```
Measure all qubits: obtain bitstring b ∈ {0,1}ⁿ
Collapse: |b⟩⟨b|
```

**Step 3 — Compute classical shadow**

```
σ = U†|b⟩⟨b|U    (shadow = classical snapshot)
```

Repeat T times → set of shadows {σ₁, σ₂, ..., σₜ}

### Observable estimation

```
Ô(ρ) ≈ (1/T) Σᵢ Tr(Oᵢ σᵢ)    (unbiased estimate)
```

---

## 4. Implementation in LightQOS

### HIO — Holographic I/O

The HIO module implements the Classical Shadows protocol:

```rust
// kernel/src/hio/shadow_copy.rs
pub struct ShadowCopy {
    num_qubits: usize,
    shadows: Vec<QuantumShadow>,
    clifford_sampler: CliffordSampler,
}

impl ShadowCopy {
    pub fn collect_shadow(&mut self, state: &QuantumState) -> QuantumShadow {
        // 1. Sample a random Clifford
        let (unitary, clifford_index) = self.clifford_sampler.sample();
        
        // 2. Apply and measure
        let rotated = state.apply(&unitary);
        let measurement = rotated.measure_computational_basis();
        
        // 3. Compute classical shadow
        let shadow = unitary.dagger() * ket(measurement) * bra(measurement) * unitary;
        
        QuantumShadow { clifford_index, measurement, shadow, fidelity_estimate: ... }
    }
}
```

### Observable estimation

```rust
// kernel/src/hio/observable_view.rs
pub fn estimate_observable(&self, observable: &PauliString) -> f64 {
    let estimates: Vec<f64> = self.shadows.iter()
        .map(|s| trace(observable * &s.shadow_matrix))
        .collect();
    
    estimates.iter().sum::<f64>() / estimates.len() as f64
}
```

---

## 5. Statistical Certificate

### Hoeffding inequality

For T shadows and observables bounded in [-1, +1]:

```
P(|Ô(ρ) - Tr(Oρ)| > ε) ≤ 2 exp(-2ε²T)
```

For T=1000 and ε=0.05: **failure probability < 2 × 10⁻²²**

### Implementation

```rust
// shadow_tomography/src/statistical_certificate.rs
pub struct StatisticalCertificate {
    pub n_shadows: usize,
    pub epsilon: f64,
    pub delta: f64,           // failure probability
    pub required_shadows: usize,
    pub success_probability: f64,
}

impl StatisticalCertificate {
    pub fn compute(n_shadows: usize, n_observables: usize, epsilon: f64) -> Self {
        let delta = 2.0 * (n_observables as f64) * (-2.0 * epsilon.powi(2) * n_shadows as f64).exp();
        let required = (2.0 * n_observables as f64 / delta).ln() / (2.0 * epsilon.powi(2)) as usize;
        
        Self {
            n_shadows,
            epsilon,
            delta,
            required_shadows: required,
            success_probability: 1.0 - delta,
        }
    }
}
```

---

## 6. Mid-Circuit Feedback

LightQOS implements **mid-circuit feedback** through HIO:

```
Execute gates 1..k
     │
  Measure qubit i (Shadow) ──→ HIO processes shadow
     │                              │
     │                    Update estimate of ρ
     │                              │
     │              Condition gates k+1..n on the result
     │
Continue circuit
```

```python
from lightqos import QuantumCircuit, MidCircuitFeedback

circuit = QuantumCircuit(4, feedback=MidCircuitFeedback())
circuit.h(0)
circuit.cnot(0, 1)

# Mid-circuit measurement and conditioning
m = circuit.mid_measure(0)   # → shadow + classical bit
circuit.x(2).if_bit(m, 1)    # X on qubit 2 if measured 1

circuit.measure([2, 3])
```

---

## 7. Adaptive Resampling

When shadow quality is low, LightQOS applies **adaptive resampling**:

```rust
// shadow_tomography/src/adaptive_resampling.rs
pub fn resample_if_needed(&mut self, threshold: f64) {
    let mean_fidelity = self.compute_mean_fidelity();
    
    if mean_fidelity < threshold {
        // Discard low-fidelity shadows
        self.shadows.retain(|s| s.fidelity_estimate > threshold * 0.8);
        
        // Collect more shadows with different Cliffords
        self.expand_clifford_pool();
        self.trigger_resample_event();
    }
}
```

---

## 8. References

1. Aaronson, S. (2019). *Shadow tomography of quantum states*. SIAM Journal on Computing, 49(5), STOC18-368.
2. Huang, H. Y., Kueng, R., & Preskill, J. (2020). *Predicting many properties of a quantum system from very few measurements*. Nature Physics, 16(10), 1050-1057.
3. Kunjummen, J., Tran, M. C., Carney, D., & Taylor, J. M. (2023). *Shadow process tomography of quantum channels*. Physical Review A, 107(4), 042403.
