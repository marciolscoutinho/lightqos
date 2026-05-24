# 📚 API Reference — EMF (Entangled Memory Fabric)

> Rust module: `lightqos::emf` | Python binding: `lightqos.EMFManager`

---

## Python Classes (PyO3)

### `EMFManager`

Main manager for the entanglement fabric. Maintains a pool of Bell pairs and applies QoS policies.

```python
import lightqos as lq

emf = lq.EMFManager(max_pairs=1000)
```

#### `EMFManager(max_pairs: int = 500)`

| Parameter   | Type  | Default | Description           |
| ----------- | ----- | ------- | --------------------- |
| `max_pairs` | `int` | `500`   | Maximum pool capacity |

#### Methods

```python
# Allocate an entangled pair
pair_id: str = emf.allocate_pair(fidelity_min: float = 0.90) -> str

# Release a pair
emf.release_pair(pair_id: str) -> bool

# Statistics for a pair
stats: dict = emf.get_pair_stats(pair_id: str) -> dict

# Global pool statistics
stats: dict = emf.pool_stats() -> dict

# Force recycling of degraded pairs
recycled: int = emf.gc() -> int
```

#### Complete Example

```python
import lightqos as lq

emf = lq.EMFManager(max_pairs=200)

# Allocate pair with minimum fidelity of 95%
pair_id = emf.allocate_pair(fidelity_min=0.95)

# Check statistics
stats = emf.get_pair_stats(pair_id)
print(stats)
# {
#   "id": "uuid-...",
#   "fidelity": 0.973,
#   "entropy": 0.021,
#   "age_ms": 1.2,
#   "status": "ACTIVE"
# }

# Use the pair for a quantum operation
# ...

# Release after use
emf.release_pair(pair_id)

# View pool state
pool = emf.pool_stats()
print(pool)
# {
#   "total_allocated": 1,
#   "active": 0,
#   "recycled": 0,
#   "mean_fidelity": 0.973
# }
```

---

### `EntangledPair`

Represents an individual entangled pair with quality metrics.

```python
pair: lq.EntangledPair = emf.get_pair_object(pair_id)
```

#### Properties (read-only)

| Property      | Type    | Description                  |
| ------------- | ------- | ---------------------------- |
| `id`          | `str`   | Pair UUID                    |
| `fidelity`    | `float` | Current fidelity (0.0-1.0)   |
| `entropy`     | `float` | von Neumann entropy          |
| `age_ms`      | `float` | Lifetime in ms               |
| `is_degraded` | `bool`  | Below the fidelity threshold |

---

## Rust Modules

### `emf::EntanglementPool`

```rust
use lightqos::emf::EntanglementPool;

let mut pool = EntanglementPool::new(1000);
let pair_id = pool.allocate(0.95)?;
let fidelity = pool.get_fidelity(&pair_id)?;
pool.release(&pair_id);
```

### `emf::PSERRouter`

Entanglement routing through the physical shortest path.

```rust
use lightqos::emf::PSERRouter;

let router = PSERRouter::new(topology);
let path = router.find_path(src_node, dst_node)?;
let hops = router.estimate_hops(&path);
```

### `emf::EntanglementRecycler`

```rust
use lightqos::emf::EntanglementRecycler;

let recycler = EntanglementRecycler::new(fidelity_threshold: 0.80);
let recycled = recycler.gc(&mut pool);
```

### `emf::FidelityMetrics`

```rust
use lightqos::emf::metrics::FidelityMetrics;

let metrics = FidelityMetrics::compute(&density_matrix);
println!("Fidelity: {}", metrics.fidelity);
println!("Entropy:  {}", metrics.von_neumann_entropy);
println!("Concurrence: {}", metrics.concurrence);
```

---

## Data Types

### `EntangledPairStatus`

```rust
pub enum EntangledPairStatus {
    Active,     // In use
    Idle,       // Available in the pool
    Degraded,   // Below the fidelity threshold
    Recycled,   // Recycled/discarded
}
```

### `QoSPolicy`

```rust
pub struct QoSPolicy {
    pub fidelity_min: f64,      // Minimum threshold (default: 0.85)
    pub max_age_ms: u64,        // Maximum lifetime (default: 1000ms)
    pub priority: u8,           // Allocation priority (1-10)
    pub auto_recycle: bool,     // Automatic recycling
}
```

---

## Errors

| Error                       | Cause                                       |
| --------------------------- | ------------------------------------------- |
| `PairNotFound(id)`          | Invalid pair ID or pair already recycled    |
| `PoolExhausted`             | Pool has no available capacity              |
| `FidelityBelowThreshold(f)` | No pair satisfies the requested threshold   |
| `AllocationTimeout`         | Timeout while waiting for an available pair |

---

## Performance

| Operation         | Latency (Rust) | Latency (Python)     |
| ----------------- | -------------- | -------------------- |
| `allocate_pair`   | ~0.5μs         | ~5μs (PyO3 overhead) |
| `release_pair`    | ~0.1μs         | ~1μs                 |
| `pool_stats`      | ~1μs           | ~10μs                |
| `gc` (1000 pairs) | ~50μs          | ~500μs               |
