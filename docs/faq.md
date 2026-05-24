# ❓ FAQ — Frequently Asked Questions

---

## General

### What is LightQOS?

LightQOS is a quantum operating system written in Rust (kernel) and Python (SDK). It differs from frameworks such as Qiskit or Cirq because it treats the quantum computer as a **real physical system** with its own abstraction layers (EFAL, EMF, TLM, HIO), rather than as a collection of logic gates.

### Does LightQOS replace Qiskit?

No — LightQOS is complementary. It includes adapters for importing Qiskit, Cirq, and PennyLane circuits. You can use Qiskit to build circuits and LightQOS to execute them with QoS guarantees and Shadow Tomography.

### Do I need a real quantum computer?

No. LightQOS includes a complete **local simulator** that emulates all components (EFAL, EMF, TLM, HIO). For real hardware, access credentials are required (IBM Quantum, IonQ, etc.).

### What is the minimum hardware requirement?

For the local simulator, any modern PC with 4 GB RAM. To compile the Rust kernel, 8 GB RAM is recommended.

---

## Installation

### Error: `maturin not found`

```bash
pip install maturin
# or
pipx install maturin
```

### Error: `Rust toolchain not found`

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update stable
```

### Error: `error[E0463]: can't find crate for 'pyo3'`

Make sure you are compiling inside the workspace:

```bash
cd lightqos/
cargo build --release  # Not directly inside kernel/
```

### Compilation takes too long

Rust compiles in `release` mode with LTO (`lto = true`), which is slower but produces much faster binaries. For development, use:

```bash
cargo build  # Without --release, much faster
```

---

## API & Usage

### How do I create a Bell State?

```python
from lightqos import QuantumCircuit

circuit = QuantumCircuit(2)
circuit.h(0)       # Hadamard on qubit 0
circuit.cnot(0, 1) # CNOT controlled by 0
circuit.measure([0, 1])

result = circuit.execute(backend="simulator", shots=1024)
print(result.counts)  # {'00': ~512, '11': ~512}
```

### How do I use temporal contracts?

```python
from lightqos import QuantumCircuit, TemporalContract

contract = TemporalContract(
    deadline_ms=200.0,
    priority=8
)

circuit = QuantumCircuit(3)
# ... add gates ...
result = circuit.execute(backend="simulator", contract=contract)
print(f"Deadline met: {result.contract_fulfilled}")
```

### How do I verify an entangled state?

```python
# Via Shadow Tomography (more efficient)
result = circuit.execute(backend="simulator", shots=2000, shadow_tomography=True)
print(f"Estimated fidelity: {result.fidelity:.3f}")
print(f"Statistical certificate: {result.statistical_certificate:.3f}")
```

### How do I import a Qiskit circuit?

```python
from qiskit import QuantumCircuit as QiskitCircuit
from lightqos.integrations import qiskit_to_lightqos

qc = QiskitCircuit(2)
qc.h(0)
qc.cx(0, 1)

lq_circuit = qiskit_to_lightqos(qc)
result = lq_circuit.execute(backend="simulator")
```

### How do I access the Rust kernel directly?

```python
import lightqos as lq

# EMF Manager
emf = lq.EMFManager(max_pairs=500)
pair_id = emf.allocate_pair(fidelity_min=0.95)
stats = emf.get_pair_stats(pair_id)

# TLM Scheduler
scheduler = lq.HarmonicScheduler(epoch_duration_us=1000)
contract = lq.TemporalContract("CNOT_GATE", deadline_ms=100.0, priority=7)
scheduled = scheduler.schedule(contract)
```

---

## Hardware Drivers

### How do I connect to IBM Quantum?

```python
from lightqos.drivers import IBMDriver

driver = IBMDriver(api_token="YOUR_IBM_TOKEN")
await driver.connect()
backends = await driver.list_backends()
print(backends)  # ['ibm_heron', 'ibm_eagle', ...]
```

### How do I use Qblox in simulation mode?

```python
from lightqos.drivers import QbloxDriver

# Simulation mode (without real hardware)
driver = QbloxDriver(simulated=True)
pulse = driver.create_x_gate(qubit_freq_hz=5.1e9, sequencer=0)
driver.send_pulse(slot=1, pulse=pulse)
driver.trigger()
```

### Does LightQOS support photonics (Xanadu)?

The photonic driver is planned for v0.3.0. Currently, the simulator supports equivalent operations.

---

## The Light AI

### Is the `the_light` module mandatory?

No. The SDK works without `the_light`. AI is optional and is activated automatically when installed.

### Where are the pre-trained models?

The `.pth` models (PyTorch) are automatically downloaded from HuggingFace Hub on first use. Approximate size: 200 MB.

### Can I train my own model?

Yes, using the scripts in `the_light/training/`:

```bash
python the_light/training/train_transpiler.py --dataset circuits_dataset.json
python the_light/training/train_demand.py --dataset emf_logs.csv
```

---

## Theory

### What is the “Ether Field”?

EFAL uses Einstein’s (1920) concept of **electromagnetic ether** as an abstract reference frame for quantum operations — it does not imply ether as a real physical medium. It is a mathematical metaphor for the 37-dimensional field that describes the hardware state.

### What are the “10 octaves”?

EFAL organises operation frequencies into 10 electromagnetic bands or “octaves” — from radio to X-ray — mapping each qubit to an operation octave with distinct physical parameters.

### What is Shadow Tomography?

A protocol proposed by Scott Aaronson (2019): instead of performing O(4ⁿ) measurements for full tomography, it uses O(log M · ε⁻²) random measurements with Clifford rotations to estimate M observables with error ε. For 50 qubits, the difference is 10⁴ vs 10³⁰ measurements.

### What is the Process Tensor?

A formalism by Milz & Modi (2021) that generalises the concept of a CPTP quantum channel to include **non-Markovian memory** — when the environment retains information between operations. It is essential for modelling real hardware with crosstalk and drift.

---

## Performance

### What is the speedup of the Rust kernel vs pure Python?

Typically 100× for numerically intensive operations such as fidelity calculation, routing, and state reconstruction. See `tests/benchmark_performance.py` for updated benchmarks.

### Is LightQOS thread-safe?

The Rust kernel is fully thread-safe. The Python SDK uses the GIL in PyO3 calls; for Python parallelism, use `asyncio` or `multiprocessing`.

---

## Contribution

### How do I report a bug?

Open a GitHub issue with:

1. LightQOS version (`python -c "import lightqos; print(lightqos.__version__)"`)
2. Rust version (`rustc --version`)
3. Operating system
4. Minimal code that reproduces the bug

### How do I propose a new feature?

Open an issue with the `enhancement` tag describing the use case and the proposed API.
