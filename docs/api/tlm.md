# 📚 API Reference — TLM (Temporal Layer Manager)

> Rust module: `lightqos::tlm` | Python bindings: `lightqos.ContractManager`, `lightqos.HarmonicScheduler`

---

## Python Classes (PyO3)

### `TemporalContract`

```python
import lightqos as lq

contract = lq.TemporalContract(
    operation="CNOT_GATE",
    deadline_ms=100.0,
    phase_ns=0,
    priority=7
)
```

| Parameter     | Type    | Default | Description                |
| ------------- | ------- | ------- | -------------------------- |
| `operation`   | `str`   | —       | Operation name             |
| `deadline_ms` | `float` | `100.0` | Deadline in milliseconds   |
| `phase_ns`    | `int`   | `0`     | Start phase in nanoseconds |
| `priority`    | `int`   | `5`     | Priority 1-10              |

#### Properties

| Property      | Type    | Description              |
| ------------- | ------- | ------------------------ |
| `id`          | `str`   | Unique UUID              |
| `operation`   | `str`   | Operation name           |
| `deadline_ms` | `float` | Deadline in ms           |
| `fulfilled`   | `bool`  | Whether it was fulfilled |

#### Methods

```python
contract.elapsed_ms() -> float         # Elapsed time
contract.is_expired() -> bool          # Whether it has expired
contract.time_remaining_ms() -> float  # Remaining margin, negative if expired
```

---

### `ContractManager`

```python
mgr = lq.ContractManager()

# Create and register contract
contract = mgr.create_contract("H_GATE", deadline_ms=50.0, priority=8)

# Mark as fulfilled
mgr.fulfill(contract.id)

# Clear expired contracts
removed = mgr.gc_expired()

# Statistics
stats = mgr.stats()
# {"total_registered": 10, "active": 2, "fulfilled": 7, "expired": 1}
```

---

### `HarmonicScheduler`

Organizes operations into **harmonic epochs**: fixed temporal windows.

```python
scheduler = lq.HarmonicScheduler(epoch_duration_us=1000)

# Advance to next epoch
epoch = scheduler.tick()

# Try to schedule a contract
ok = scheduler.schedule(contract)  # False if expired

# Statistics
stats = scheduler.stats()
# {"epoch": 5, "epoch_duration_us": 1000, "scheduled": 42, "dropped": 2}
```

---

## Rust Modules

### `tlm::HarmonicScheduler`

```rust
use lightqos::tlm::HarmonicScheduler;

let mut scheduler = HarmonicScheduler::new(Duration::from_micros(1000));
scheduler.tick();
let slot = scheduler.next_available_slot(priority)?;
```

### `tlm::TemporalContract`

```rust
use lightqos::tlm::{TemporalContract, Phase};

let contract = TemporalContract::new(
    "MEASURE_QUBIT_0",
    Duration::from_millis(200),
    Phase::ns(500),
    Priority::HIGH,
);
assert!(!contract.is_expired());
```

### `tlm::ProcessTensor`

Memory for non-Markovian channels:

```rust
use lightqos::tlm::ProcessTensor;

let mut pt = ProcessTensor::new(order: 3);
pt.add_step(channel_matrix);
let non_markov_coeff = pt.non_markovianity_measure();
```

### `tlm::Snapshot`

```rust
use lightqos::tlm::Snapshot;

let snap = Snapshot::capture(&current_state);
// ... potentially failed operation ...
if operation_failed {
    snap.restore(&mut current_state);
}
```

---

## Errors

| Error                 | Cause                         |
| --------------------- | ----------------------------- |
| `ContractExpired(id)` | Deadline exceeded             |
| `SchedulerFull`       | All epoch slots occupied      |
| `InvalidPriority(p)`  | Priority outside [1, 10]      |
| `RollbackFailed`      | Invalid or corrupted snapshot |

---

## Example: Complete TLM Pipeline

```python
import lightqos as lq
import time

mgr = lq.ContractManager()
sched = lq.HarmonicScheduler(epoch_duration_us=500)

# Create contract for gate sequence
c = mgr.create_contract("BELL_STATE", deadline_ms=300.0, priority=9)

# Schedule
if not sched.schedule(c):
    print("Contract expired before scheduling!")
else:
    # Execute operation
    time.sleep(0.05)  # simulates 50ms of execution

    # Fulfill contract
    mgr.fulfill(c.id)
    print(f"✅ Completed in {c.elapsed_ms():.1f}ms (deadline: {c.deadline_ms}ms)")

print(mgr.stats())
```
