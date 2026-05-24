# 📚 API Reference — The Light AI

> Python module: `the_light` | Requires: PyTorch >= 2.0

---

## Installation

```bash
pip install -e the_light/
# or with ML dependencies:
pip install "lightqos[ai]"
```

---

## `TranspilerOptimizer`

Transformer ML-based circuit optimizer. Reduces the number of gates while preserving unitary equivalence.

```python
from the_light import TranspilerOptimizer

optimizer = TranspilerOptimizer(
    model_path=None,    # None = downloads pretrained model
    device="cpu"        # "cpu" | "cuda" | "mps"
)
```

### Methods

```python
# Optimize a circuit for a specific backend
optimized = optimizer.optimize(
    circuit,
    target_backend="ibm_heron",   # ibm_heron | ionq_forte | simulator
    optimization_level=2,          # 1-3
    max_iterations=1000
)

print(f"Gates before: {circuit.num_gates()}")
print(f"Gates after: {optimized.num_gates()}")
print(f"Depth before: {circuit.depth()}")
print(f"Depth after: {optimized.depth()}")

# Estimate optimization quality
score = optimizer.quality_score(circuit, optimized)
print(f"Score: {score:.3f}")  # 0.0-1.0

# Verify unitary equivalence
is_equivalent = optimizer.verify_equivalence(circuit, optimized)
assert is_equivalent
```

### Supported Backends

| Backend      | Native Gate Set | Connectivity |
| ------------ | --------------- | ------------ |
| `ibm_heron`  | ECR, X, SX, Rz  | Heavy-hex    |
| `ionq_forte` | GPI, GPI2, MS   | All-to-all   |
| `simulator`  | Universal       | All-to-all   |

---

## `EMFPredictor`

Entangled-pair demand forecasting using LSTM. Proactively creates pairs before they are needed.

```python
from the_light import EMFPredictor

predictor = EMFPredictor(
    horizon_ms=500,          # Forecast horizon
    lookback_ms=2000,        # Historical observation window
    model_type="lstm"        # "lstm" | "transformer"
)
```

### Methods

```python
# Forecast demand for the next N ms
forecast = predictor.forecast(horizon_ms=500)
print(forecast)
# {
#   "timestamp_ms": [0, 50, 100, ...],
#   "predicted_pairs": [12, 15, 11, ...],
#   "confidence": [0.94, 0.91, 0.88, ...],
#   "peak_demand": 18,
#   "peak_at_ms": 250
# }

# Update model with real data
predictor.update(observed_demand=[10, 12, 15, 9])

# Recommend preallocation to EMF
recommendation = predictor.recommend_preallocation()
print(f"Preallocate {recommendation['pairs']} pairs now")
```

---

## `ConsciousnessMath`

18-dimensional mathematical engine based on Tononi's Integrated Information Theory (IIT).

```python
from the_light import ConsciousnessMath

math = ConsciousnessMath(dimensions=18)
```

### Methods

```python
# Compute Φ (phi) — measure of information integration
phi = math.compute_phi(density_matrix)
print(f"Φ = {phi:.4f}")  # >0 indicates integrated consciousness

# Map quantum state into 18D space
state_18d = math.embed(quantum_state)  # ndarray (18,)

# Distance in consciousness space
distance = math.consciousness_distance(state_a, state_b)

# TUCU correlation kernel
correlation = math.tucu_kernel(state_a, state_b)
```

---

## `Transformer` (circuit → sequence)

Transformer module for processing circuit sequences as a language.

```python
from the_light import CircuitTransformer

transformer = CircuitTransformer(
    vocab_size=128,      # Gates as tokens
    d_model=512,
    n_heads=8,
    n_layers=6
)

# Encode circuit as a token sequence
tokens = transformer.encode(circuit)

# Generate equivalent optimized circuit
optimized_tokens = transformer.generate(tokens, max_length=200)
optimized_circuit = transformer.decode(optimized_tokens)
```

---

## `AdaptiveCalibration`

Real-time adaptive calibration of gate parameters.

```python
from the_light import AdaptiveCalibration

calibrator = AdaptiveCalibration(backend_driver=driver)

# Calibrate all qubits
results = await calibrator.calibrate_all()
for qubit_id, params in results.items():
    print(f"Qubit {qubit_id}: freq={params['frequency_hz']:.3e}, T1={params['T1_us']:.1f}μs")

# Incremental calibration: only changed qubits
delta = await calibrator.incremental_update()
print(f"Updated: {delta['updated_qubits']} qubits")
```

---

## Constants and Types

```python
from the_light.types import OptimizationLevel, BackendTarget

# Optimization levels
OptimizationLevel.LIGHT    # 1 — fast, minimal reduction
OptimizationLevel.BALANCED # 2 — balance (default)
OptimizationLevel.DEEP     # 3 — slow, maximum reduction

# Available targets
BackendTarget.IBM_HERON
BackendTarget.IONQ_FORTE
BackendTarget.SIMULATOR
```

---

## Errors

| Error                | Cause                                                 |
| -------------------- | ----------------------------------------------------- |
| `ModelNotFound`      | Pretrained model not found and no internet connection |
| `EquivalenceError`   | Optimization produced a different unitary             |
| `CalibrationTimeout` | Hardware did not respond within the timeout           |
| `InsufficientData`   | Not enough historical data for forecasting            |
