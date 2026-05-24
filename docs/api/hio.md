# 📚 API Reference — HIO (Holographic I/O)

> Rust module: `lightqos::hio` | Python bindings: `lightqos.ShadowCollector`, `lightqos.QuantumShadow`

---

## Python Classes (PyO3)

### `ShadowCollector`

Accumulates quantum shadows and reconstructs the state through Shadow Tomography.

```python
import lightqos as lq

collector = lq.ShadowCollector(
    num_qubits=4,
    target_shadows=1000,
    reconstruction_threshold=0.85
)
```

| Parameter                  | Type    | Default | Description                          |
| -------------------------- | ------- | ------- | ------------------------------------ |
| `num_qubits`               | `int`   | —       | Number of system qubits              |
| `target_shadows`           | `int`   | `1000`  | Shadows required for reconstruction  |
| `reconstruction_threshold` | `float` | `0.85`  | Minimum fidelity for the certificate |

#### Methods

```python
# Add existing shadow
collector.add_shadow(shadow: QuantumShadow)

# Measure and create shadow (bits + Clifford index)
shadow = collector.measure(bits=[0, 1, 0, 1], clifford_index=42)

# Progress
pct = collector.progress_pct()   # 0.0-100.0
ready = collector.ready_for_reconstruction()

# Reconstruct state
result = collector.reconstruct()
# {
#   "success": True,
#   "num_shadows": 1000,
#   "mean_fidelity": 0.924,
#   "statistical_certificate": 0.891,
#   "reconstruction_quality": "HIGH"
# }

# Statistics
stats = collector.stats()

# Clear
collector.reset()
```

---

### `QuantumShadow`

Individual quantum shadow — the result of a random classical measurement.

```python
shadow = lq.QuantumShadow(
    num_qubits=3,
    measurement_bits=[0, 1, 0],
    clifford_index=17
)
```

#### Properties

| Property            | Type        | Description                            |
| ------------------- | ----------- | -------------------------------------- |
| `id`                | `str`       | UUID                                   |
| `num_qubits`        | `int`       | Number of qubits                       |
| `measurement_bits`  | `list[int]` | Measured bits (0 or 1)                 |
| `clifford_index`    | `int`       | Index of the applied Clifford rotation |
| `fidelity_estimate` | `float`     | Fidelity estimate                      |

```python
shadow.bitstring()  # -> "010"
```

---

## Rust Modules

### `hio::ShadowCopy`

```rust
use lightqos::hio::ShadowCopy;

let mut shadow_copy = ShadowCopy::new(n_qubits: 4);
let shadow = shadow_copy.collect_shadow(&clifford_unitary)?;
let observable_estimate = shadow_copy.estimate_observable(&pauli_string)?;
```

### `hio::ObservableView`

```rust
use lightqos::hio::ObservableView;

let view = ObservableView::from_shadows(&shadows);
let expval = view.expectation_value(&pauli_zz)?;
let error_bound = view.error_bound(epsilon: 0.01, delta: 0.05);
```

### `hio::StatisticalGuarantee`

```rust
use lightqos::hio::StatisticalGuarantee;

let guarantee = StatisticalGuarantee::compute(
    n_shadows: 1000,
    n_observables: 100,
    epsilon: 0.01,
);
println!("Success probability: {}", guarantee.success_probability);
println!("Required shadows: {}", guarantee.required_shadows);
```

---

## Complexity

| Method                | Measurements       | Storage          |
| --------------------- | ------------------ | ---------------- |
| Full tomography       | O(4ⁿ)              | O(4ⁿ)            |
| **Shadow Tomography** | **O(log M · ε⁻²)** | **O(n · log M)** |

For n=50 qubits, M=1000 observables, ε=0.01:

- Full tomography: **10³⁰ measurements** ❌
- Shadow Tomography: **~46,000 measurements** ✅

---

## Complete Example

```python
import lightqos as lq
import random

# Setup
collector = lq.ShadowCollector(num_qubits=3, target_shadows=500)

# Simulate shadow collection.
# On real hardware, this would come from the device.
for i in range(500):
    bits = [random.randint(0, 1) for _ in range(3)]
    clifford_idx = random.randint(0, 2**16 - 1)
    collector.measure(bits, clifford_idx)

print(f"Progress: {collector.progress_pct():.0f}%")
print(f"Mean fidelity: {collector.mean_fidelity():.3f}")

# Reconstruct
result = collector.reconstruct()
if result["success"]:
    print(f"Quality: {result['reconstruction_quality']}")
    print(f"Certificate: {result['statistical_certificate']:.3f}")
```
