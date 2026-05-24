# 🏗️ LightQOS Architecture

> **Version**: 0.2.0 | **Date**: 2026

---

## Overview

LightQOS is organised into **four vertical abstraction layers**, a cross-cutting AI module, and hardware drivers:

```
┌─────────────────────────────────────────────────────────────┐
│                    PYTHON SDK (frontend/)                   │
│          QuantumCircuit · TemporalContract · Adapters       │
├─────────────────────────────────────────────────────────────┤
│                   THE LIGHT AI (the_light/)                 │
│     Transpiler · EMF Predictor · Calibration · 18D Math     │
├──────────────┬──────────────┬──────────────┬────────────────┤
│     EFAL     │     EMF      │     TLM      │      HIO       │
│  Ether Field │  Entangled   │   Temporal   │  Holographic   │
│  Abstraction │   Memory     │    Layer     │     I/O        │
│    Layer     │    Fabric    │   Manager    │                │
├──────────────┴──────────────┴──────────────┴────────────────┤
│               RUST KERNEL (kernel/) — PyO3                  │
│    math/ · process_tensor/ · shadow_tomography/ · protocols/ │
├─────────────────────────────────────────────────────────────┤
│                   DRIVERS (drivers/src/)                    │
│  IBM · IonQ · Qblox · Zurich Instruments · Simulator       │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. EFAL — Ether Field Abstraction Layer

### Concept

EFAL treats quantum hardware as a **continuous physical field** — the Ether Field — rather than a discrete collection of qubits. It is inspired by Einstein’s electromagnetic ether (1920) as a reference frame for operations.

### Internal structure

```
kernel/src/efal/
├── mod.rs          # Public interface and dynamic topology
├── geometry.rs     # Geometric Algebra GA(3,0) — 10 EM octaves
├── channel.rs      # CPTP quantum channels
├── defect.rs       # Topological defects (qubit representation)
└── field_driver.rs # Field drivers for real hardware
```

### Responsibilities

- Represent qubits as **topological defects** in a 37-dimensional Ether field
- Manage **10 electromagnetic octaves** as operation bands
- Provide hardware-independent **CPTP quantum channel** abstraction
- Orchestrate **dynamic topology** at runtime

---

## 2. EMF — Entangled Memory Fabric

### Concept

EMF manages **entanglement as a scarce resource** — it creates, distributes, monitors, and recycles Bell pairs with fidelity and QoS guarantees.

### Internal structure

```
kernel/src/emf/
├── mod.rs              # Public interface
├── entanglement_pool.rs # Bell pair pool
├── pser_routing.rs     # PSER routing (Physical-Shortest-Entanglement-Route)
├── metrics.rs          # Fidelity and entropy metrics
└── recycler.rs         # Recycling of degraded pairs
```

### Entanglement pipeline

```
Pair request  →  Pool (hit?) ──── YES ──→  Direct allocation
                    │
                    NO
                    │
             PSER routing  →  Pair creation  →  Allocation
                                    │
                            Fidelity monitoring
                                    │
                            Fidelity < threshold?
                                │         │
                               YES        NO
                                │         │
                            Recycling   Continue
```

---

## 3. TLM — Temporal Layer Manager

### Concept

TLM ensures that quantum operations **respect physical time windows** — coherence, heralding, laser synchronisation. It uses formal contracts with deadlines and automatic rollback.

### Internal structure

```
kernel/src/tlm/
├── mod.rs               # Public interface
├── contract.rs          # Temporal contracts (SLA)
├── harmonic_scheduler.rs # Harmonic-epoch scheduler
├── process_tensor.rs    # Non-Markovian channel memory
└── snapshot.rs          # Context snapshots for rollback
```

### Contract lifecycle

```
create_contract(op, deadline_ms, priority)
         │
    [Scheduling]  ←── HarmonicScheduler
         │
    [Execution]  ─────────────────────────────────────────┐
         │                                                │
    [Success]  → fulfill(id) → contract closed     [Failure/Timeout]
                                                          │
                                                   rollback(snapshot)
```

---

## 4. HIO — Holographic I/O

### Concept

HIO uses **Shadow Tomography** (Aaronson, 2019) to reconstruct quantum states from random classical measurements. It is far more efficient than full tomography.

### Internal structure

```
kernel/src/hio/
├── mod.rs                  # Public interface
├── shadow_copy.rs          # Quantum shadow collection
├── observable_view.rs      # Observable estimation
└── statistical_guarantee.rs # Statistical confidence certificates

shadow_tomography/src/
├── lib.rs
├── adaptive_resampling.rs  # Adaptive resampling
├── mid_circuit_feedback.rs # Mid-circuit feedback
├── observable_view.rs
├── shadow_copy.rs
└── statistical_certificate.rs
```

### Complexity

| Method            | Required samples | Storage      |
| ----------------- | ---------------- | ------------ |
| Full tomography   | O(4ⁿ)            | O(4ⁿ)        |
| Shadow Tomography | O(log M · ε⁻²)   | O(n · log M) |

For 50 qubits and M=1000 observables: **10⁴ vs 10³⁰ measurements**.

---

## 5. The Light AI

### Components

```
the_light/
├── core.py         # Main engine — orchestration
├── optimizer.py    # TranspilerOptimizer (ML-based gate synthesis)
├── predictor.py    # EMFPredictor (LSTM demand forecasting)
└── transformer.py  # Transformer for circuit sequences
```

| Component           | Technology            | Function                               |
| ------------------- | --------------------- | -------------------------------------- |
| TranspilerOptimizer | Transformer (PyTorch) | Reduces gate count for target hardware |
| EMFPredictor        | LSTM                  | Predicts demand for entangled pairs    |
| ConsciousnessMath   | 18D Tensor            | Information integration metrics (IIT)  |

---

## 6. Process Tensor Framework

```
process_tensor/src/
├── lib.rs
├── process_tensor.rs  # General process tensor
├── memory_kernel.rs   # Non-Markovian memory kernel
├── quantum_channel.rs # Generalised CPTP channels
└── quantum_comb.rs    # Quantum combs
```

It is based on the **Milz & Modi (2021)** formalism for quantum channels with memory, enabling the modelling of non-Markovian effects in real hardware.

---

## 7. Hardware Drivers

### Driver architecture

All drivers implement the common interface defined in `drivers/src/mod.rs`:

```rust
#[async_trait]
pub trait QuantumDriver: Send + Sync {
    async fn connect(&mut self) -> Result<(), DriverError>;
    async fn execute_circuit(&self, circuit: &QuantumCircuit) -> Result<ExecutionResult, DriverError>;
    async fn get_backend_info(&self) -> BackendInfo;
    async fn disconnect(&mut self);
}
```

### Capability matrix

| Driver     | Qubits | Connectivity | Gate type    | Readout      |
| ---------- | ------ | ------------ | ------------ | ------------ |
| IBM Heron  | 133    | Heavy-hex    | Native (ECR) | Dispersive   |
| IonQ Forte | 36     | All-to-all   | Native (MS)  | Fluorescence |
| Qblox      | ∞      | Configurable | Pulses       | Homodyne     |
| Zurich     | ∞      | Configurable | Pulses       | Heterodyne   |
| Simulator  | ∞      | All-to-all   | Universal    | Perfect      |

---

## 8. Full Execution Flow

```
Python SDK
    │
    ▼
QuantumCircuit.execute(backend="ibm_heron", shots=1024)
    │
    ▼ (1) The Light AI — circuit optimisation
TranspilerOptimizer.optimize() → reduced native circuit
    │
    ▼ (2) TLM — temporal contract creation
ContractManager.create_contract(deadline_ms=500)
    │
    ▼ (3) EMF — allocation of required entangled pairs
EntanglementPool.allocate(n_pairs=..., fidelity_min=0.95)
    │
    ▼ (4) EFAL — mapping to physical topology
EtherField.map_circuit(circuit, topology=backend.topology)
    │
    ▼ (5) Driver — execution on real hardware
IBMDriver.execute(pulses) → raw measurements
    │
    ▼ (6) HIO — state reconstruction
ShadowCollector.reconstruct() → density_matrix + certificate
    │
    ▼ (7) TLM — contract closure
ContractManager.fulfill(contract_id)
    │
    ▼
ExecutionResult(counts, density_matrix, fidelity, certificate)
```

---

## 9. Rust Workspace

The project uses a **Cargo workspace** with 8 independent crates:

```toml
[workspace]
members = [
    "kernel",          # Core EFAL/EMF/TLM/HIO + PyO3 bindings
    "drivers",         # Hardware drivers
    "cli",             # CLI tool
    "math",            # Advanced mathematics (standalone)
    "process_tensor",  # Process Tensor Framework
    "shadow_tomography", # Advanced Shadow Tomography
    "simulators",      # Simulators
    "protocols",       # Quantum protocols
]
```

### Dependencies between crates

```
kernel ──→ math
drivers ──→ math
process_tensor ──→ math
shadow_tomography ──→ math
simulators ──→ math
protocols ──→ math
cli ──→ kernel
```

---

## 10. Performance Considerations

| Operation                            | Pure Python | Rust (PyO3) | Speedup |
| ------------------------------------ | ----------- | ----------- | ------- |
| EMF pair allocation                  | ~50μs       | ~0.5μs      | 100×    |
| Fidelity calculation                 | ~200μs      | ~2μs        | 100×    |
| Shadow reconstruction (1000 shadows) | ~500ms      | ~5ms        | 100×    |
| PSER routing (100 nodes)             | ~10ms       | ~100μs      | 100×    |
