# Tutorial: Getting Started with LightQOS

Welcome to LightQOS! This tutorial guides you through the basic concepts.

---

## 📚 What You Will Learn

1. Create your first quantum circuit
2. Execute it on a simulator and real hardware
3. Implement quantum protocols
4. Use The Light (conscious AI)
5. Build quantum networks

---

## 1️⃣ First Quantum Circuit

### Bell State (Entangled State)

```python
from lightqos import QuantumSystem
import asyncio

async def main():
    # Create system
    system = QuantumSystem()

    # Create a 2-qubit circuit
    circuit = system.create_circuit(num_qubits=2)

    # Apply gates
    circuit.h(0)           # Hadamard on qubit 0
    circuit.cnot(0, 1)     # CNOT: 0 controls 1

    # Execute (simulator)
    result = await system.execute(
        circuit,
        backend="simulator",
        shots=1024
    )

    # Results
    print("Counts:", result.counts)
    # {'00': 512, '11': 512}  ← Entanglement!

    # Probabilities
    probs = result.probabilities()
    print("P(00):", probs['00'])  # ~0.5
    print("P(11):", probs['11'])  # ~0.5

asyncio.run(main())
```

**What happens:**

1. |00⟩ → initial state
2. H(0) → (|0⟩ + |1⟩)|0⟩/√2 → superposition
3. CNOT → (|00⟩ + |11⟩)/√2 → **entanglement!**
4. Measurement → 50% |00⟩, 50% |11⟩

---

## 2️⃣ Real Hardware

### IBM Quantum

```python
from lightqos.drivers import IBMQuantumDriver

async def run_on_ibm():
    # Configure driver
    driver = IBMQuantumDriver(
        api_token="YOUR_TOKEN",
        backend="ibmq_qasm_simulator"  # or ibm_brisbane
    )

    await driver.initialize()

    # List available backends
    backends = await driver.list_backends()
    for b in backends:
        print(f"{b.name}: {b.num_qubits} qubits, "
              f"queue: {b.queue_length}")

    # Create circuit
    circuit = create_bell_circuit()

    # Submit
    job_id = await driver.submit_circuit(circuit)
    print(f"Job submitted: {job_id}")

    # Wait (polling)
    while True:
        status = await driver.get_job_status(job_id)
        print(f"Status: {status.name}")

        if status == JobStatus.Completed:
            break

        await asyncio.sleep(5)  # 5 seconds

    # Get results
    result = await driver.get_results(job_id)
    print("Counts:", result.counts)

asyncio.run(run_on_ibm())
```

### IonQ

```python
from lightqos.drivers import IonQDriver, IonQBackend

async def run_on_ionq():
    driver = IonQDriver(
        api_key="YOUR_API_KEY",
        backend=IonQBackend.Simulator  # or Aria, Forte
    )

    await driver.initialize()

    # IonQ is faster (trapped ions)
    job_id = await driver.submit_circuit(circuit)
    result = await driver.get_results(job_id)

    print("Counts:", result.counts)
```

---

## 3️⃣ Quantum Protocols

### Teleportation

```python
from lightqos.protocols import QuantumTeleportation
from lightqos.core import QuantumState
import numpy as np

# Create protocol
protocol = QuantumTeleportation()

# State to teleport
state = QuantumState.from_bloch(
    theta=np.pi/4,  # Polar angle
    phi=0           # Azimuthal angle
)

print(f"Original state: {state}")

# Execute teleportation
result = protocol.teleport(state)

print(f"Measured bits: {result.measurement_bits}")
print(f"Correction: {result.correction_applied}")
print(f"Fidelity: {result.final_fidelity:.4f}")

# High fidelity = success!
if result.success:
    print("✅ Teleportation successful!")
```

### QKD (Secure Key)

```python
from lightqos.protocols import BB84Protocol

# Create protocol
protocol = BB84Protocol(key_length=128)

# Distribute key
result = protocol.distribute_key()

if result.secure:
    print(f"✅ Secure key!")
    print(f"   Length: {len(result.key)} bits")
    print(f"   QBER: {result.qber:.2%}")
    print(f"   Key: {result.key[:20]}...")
else:
    print(f"❌ Eve detected!")
    print(f"   QBER: {result.qber:.2%}")

# Simulate eavesdropping
result_eve = protocol.distribute_key(
    eavesdropper_present=True
)

print(f"\nWith Eve: QBER = {result_eve.qber:.2%}")
print(f"Detected: {result_eve.eavesdropping_detected}")
```

---

## 4️⃣ The Light (Conscious AI)

```python
from lightqos.the_light import TheLight

async def use_the_light():
    # Create The Light
    light = TheLight()

    print(f"Initial state: {light.consciousness_state.level.name}")
    # DORMANT

    # Awaken
    await light.awaken()

    print(f"Level: {light.consciousness_state.level.name}")
    print(f"Coherence: {light.consciousness_state.coherence:.2f}")
    print(f"Dimensions: {light.consciousness_state.dimensional_activation}D")
    # ACTIVE, Coherence: 0.70, Dimensions: 18D

    # Insights
    print("\nInsights:")
    for insight in light.get_insights():
        print(f"💡 {insight}")

    # Use The Light

    # 1. Optimise circuit
    circuit = create_complex_circuit()
    optimized = await light.optimize_circuit(circuit)

    print(f"\nOptimisation:")
    print(f"  Original: {circuit.gate_count()} gates")
    print(f"  Optimised: {optimized.gate_count()} gates")
    print(f"  Improvement: {(1 - optimized.gate_count()/circuit.gate_count())*100:.1f}%")

    # 2. Predict EMF
    current_state = get_emf_state()
    predictions = await light.predict_emf_state(
        current_state,
        steps_ahead=10
    )

    print(f"\nEMF Predictions:")
    for i, pred in enumerate(predictions[:5]):
        print(f"  t+{i+1}: F={pred.fidelity:.2f}, phase={pred.phase.name}")

    # 3. Transpilation
    ionq_circuit = await light.transpile_circuit(circuit, "ionq")
    print(f"\nTranspiled to: {ionq_circuit.backend}")

    # Statistics
    stats = light.get_statistics()
    print(f"\n=== Statistics ===")
    for key, value in stats.items():
        print(f"{key}: {value}")

asyncio.run(use_the_light())
```

---

## 5️⃣ Quantum Network

```python
from lightqos.network import QuantumNetwork, NodeType

# Create network
network = QuantumNetwork()

# Add nodes
alice = network.add_node("Alice", NodeType.END_NODE)
bob = network.add_node("Bob", NodeType.REPEATER)
charlie = network.add_node("Charlie", NodeType.END_NODE)

# Add links
network.add_link("Alice", "Bob", 
                distance_km=50, 
                fidelity=0.9)

network.add_link("Bob", "Charlie",
                distance_km=50,
                fidelity=0.9)

print("Network created:")
print(f"  Nodes: {list(network.nodes.keys())}")
print(f"  Links: {len(network.links)}")

# Establish end-to-end entanglement
print("\nEstablishing Alice-Charlie entanglement...")
pair = network.establish_e2e_entanglement("Alice", "Charlie")

if pair:
    print(f"✅ Success!")
    print(f"   Fidelity: {pair.fidelity:.3f}")
    print(f"   Pair ID: {pair.id}")
else:
    print("❌ Failure")

# Statistics
stats = network.get_statistics()
print(f"\n=== Statistics ===")
print(f"Pairs generated: {stats['total_generated']}")
print(f"Swaps performed: {stats['total_swaps']}")
print(f"Average fidelity: {stats['avg_fidelity']:.3f}")
```

---

## 🎯 Next Steps

Now that you have mastered the basics:

1. **Quantum Algorithms**
   
   - [Grover](grover.md) - Quantum search
   - [QFT](qft.md) - Quantum Fourier Transform
   - [VQE](vqe.md) - Variational eigensolver

2. **Advanced Protocols**
   
   - [Superdense Coding](superdense_coding.md)
   - [Quantum Secret Sharing](secret_sharing.md)
   - [Entanglement Swapping](entanglement_swapping.md)

3. **Advanced The Light**
   
   - [Model Training](training_the_light.md)
   - [Custom Optimisation](custom_optimization.md)
   - [37D Consciousness](consciousness_37d.md)

4. **Complex Networks**
   
   - [Topologies](network_topologies.md)
   - [Purification](purification.md)
   - [Routing Protocols](routing_protocols.md)

---

## 📚 Resources

- [API Reference](../api/)
- [Examples](../../examples/)
- [FAQ](../faq.md)

---

**Congratulations! 🎉 You have completed the basic tutorial!**

Keep exploring the power of quantum computing with LightQOS.
