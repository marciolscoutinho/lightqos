# 📚 API Reference — Hardware Drivers

> Rust module: `lightqos_drivers` | Python wrapper: `lightqos.drivers`

---

## Common Interface

All drivers implement the `QuantumDriver` trait:

```rust
#[async_trait]
pub trait QuantumDriver: Send + Sync {
    async fn connect(&mut self) -> Result<(), DriverError>;
    async fn execute_circuit(&self, circuit: &QuantumCircuit) -> Result<ExecutionResult, DriverError>;
    async fn get_backend_info(&self) -> BackendInfo;
    async fn calibrate(&mut self) -> Result<CalibrationResult, DriverError>;
    async fn disconnect(&mut self);
}
```

---

## IBM Quantum Driver (`ibm_driver.rs`)

Driver for IBM Quantum systems: Heron, Eagle and Falcon.

```python
from lightqos.drivers import IBMDriver

driver = IBMDriver(
    api_token="IBM_QUANTUM_TOKEN",
    backend="ibm_heron",
    hub="ibm-q",
    group="open",
    project="main"
)

await driver.connect()
info = await driver.get_backend_info()
# {"name": "ibm_heron", "qubits": 133, "topology": "heavy_hex", "T1_mean_us": 250}

result = await driver.execute_circuit(circuit, shots=1024)
print(result.counts)  # {"00": 512, "11": 512}

await driver.disconnect()
```

### IBM Heron Characteristics

| Parameter          | Value                        |
| ------------------ | ---------------------------- |
| Qubits             | 133                          |
| Topology           | Heavy-hex                    |
| Native gate        | ECR (Echoed Cross-Resonance) |
| Mean T1            | ~250 μs                      |
| Mean T2            | ~130 μs                      |
| Gate fidelity (2Q) | ~99.5%                       |

### Simulation Mode

```python
driver = IBMDriver(simulated=True, n_qubits=27)
```

---

## IonQ Driver (`ionq_driver.rs`)

Driver for IonQ Forte via cloud API.

```python
from lightqos.drivers import IonQDriver

driver = IonQDriver(
    api_key="IONQ_API_KEY",
    backend="ionq_forte",   # ionq_forte | ionq_aria
    noise_model="ideal"     # ideal | realistic
)

await driver.connect()
result = await driver.execute_circuit(circuit, shots=2000)
```

### IonQ Forte Characteristics

| Parameter                | Value                           |
| ------------------------ | ------------------------------- |
| Qubits                   | 36                              |
| Topology                 | All-to-all                      |
| Native gates             | GPI, GPI2, MS (Mølmer-Sørensen) |
| #AQ (Algorithmic Qubits) | 35                              |
| Gate fidelity (2Q)       | ~99.9%                          |

---

## Qblox Driver (`qblox_driver.rs`)

Driver for Qblox hardware using microwave pulses.

```python
from lightqos.drivers import QbloxDriver

driver = QbloxDriver(
    cluster_ip="192.168.1.100",
    simulated=False
)

# Configure standard 6-qubit cluster
driver.configure_standard_setup()

# Create and send X-gate pulse
pulse = driver.create_x_gate(
    qubit_freq_hz=5.1e9,
    sequencer=0,
    slot=1
)
driver.send_pulse(pulse)
driver.trigger()
```

### Module Types

| Module | Function              | Channels           |
| ------ | --------------------- | ------------------ |
| QCM    | Control / drive       | 4 IQ outputs       |
| QCM-RF | Integrated RF control | 2 RF outputs       |
| QRM    | Readout               | 1 input + 1 output |
| QRM-RF | Integrated RF readout | 1 RF pair          |

### Available Pulses

```python
from lightqos.drivers.qblox import QbloxPulse

# Rectangular
p = QbloxPulse.rectangular(seq=0, lo_freq=5e9, nco_freq=0, amp=0.5, duration_ns=40)

# Gaussian
p = QbloxPulse.gaussian(seq=0, lo_freq=5e9, nco_freq=0, amp=0.5, sigma_ns=8, duration_ns=40)

# DRAG (reduces leakage)
p = QbloxPulse.drag(seq=0, lo_freq=5e9, nco_freq=0, amp=0.5, sigma_ns=8, beta=0.5, duration_ns=40)

# Predefined gates
p = QbloxPulse.x_gate(qubit_freq_hz=5.1e9, sequencer=0)
p = QbloxPulse.h_gate(qubit_freq_hz=5.1e9, sequencer=0)
```

---

## Zurich Instruments Driver (`zurich_driver.rs`)

Driver for Zurich Instruments SHFQA/SHFSG.

```python
from lightqos.drivers import ZurichDriver

# 17-qubit setup
setup = ZurichDriver.configure_17_qubit_setup()

# Send pulse to SHFSG-001
pulse = ZIPulse.gaussian(
    channel=0,
    center_freq=5.0e9,
    mod_freq=0.0,
    amplitude=0.8,
    sigma_ns=8.0,
    duration_ns=40
)
setup.send_pulse("SHFSG-001", pulse)
setup.trigger_all()

# Readout through SHFQA
prob_0 = setup.measure_qubit("SHFQA-001", channel=0, fidelity=0.99)
print(f"P(|0⟩) = {prob_0:.3f}")
```

### Typical Setup

```
PQSC (global synchronization)
  ├── SHFSG-001 (drive, 8 channels) → qubits 0-7
  ├── SHFSG-002 (drive, 8 channels) → qubits 8-15
  └── SHFQA-001 (readout, 4 channels) → resonators 0-3
```

---

## Simulator Driver (`simulator_driver.rs`)

Local state-vector simulator.

```python
from lightqos.drivers import SimulatorDriver

driver = SimulatorDriver(
    n_qubits=20,            # Up to ~30, limited by memory
    noise_model=None,       # None = ideal; or NoiseModel(...)
    seed=42
)

result = await driver.execute_circuit(circuit, shots=8192)
print(result.statevector)   # complex numpy array (2^n,)
print(result.probabilities) # real numpy array (2^n,)
```

### Noise Models

```python
from lightqos.drivers.simulator import NoiseModel

noise = NoiseModel(
    t1_us=100.0,            # Relaxation
    t2_us=80.0,             # Decoherence
    gate_error_1q=0.001,    # 1-qubit gate error
    gate_error_2q=0.01,     # 2-qubit gate error
    readout_error=0.02      # Readout error
)
driver = SimulatorDriver(n_qubits=5, noise_model=noise)
```

---

## Common Types

```python
from lightqos.drivers.types import BackendInfo, ExecutionResult

# BackendInfo
info.name         # str
info.n_qubits     # int
info.topology     # dict
info.gate_set     # list[str]
info.is_simulator # bool

# ExecutionResult
result.counts         # dict[str, int]  e.g. {"00": 512, "11": 512}
result.shots          # int
result.duration_ms    # float
result.fidelity       # float | None (if shadow tomography is active)
```

---

## Errors

| Error                   | Cause                                         |
| ----------------------- | --------------------------------------------- |
| `ConnectionFailed(msg)` | Network failure or invalid credentials        |
| `BackendUnavailable`    | Hardware under maintenance or queue full      |
| `CircuitTooLarge(n)`    | Circuit exceeds backend capacity              |
| `CalibrationFailed`     | Hardware did not calibrate within the timeout |
| `InvalidPulseParams`    | Pulse parameters outside hardware limits      |
