# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# qiskit_integration.py — Qiskit Integration — importing Qiskit circuits into LightQOS
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 25-08-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Example: Complete Qiskit integration
Demonstrates how to use LightQOS with existing Qiskit circuits
"""

from qiskit import QuantumCircuit as QiskitCircuit
from qiskit import transpile
from qiskit_aer import AerSimulator
import time

try:
    from lightqos.integrations import qiskit_to_lightqos, lightqos_to_qiskit
    from lightqos import QuantumCircuit as LightQOSCircuit
    from lightqos import TemporalContract
except ImportError:
    print("⚠️  LightQOS not installed. Run: pip install -e ../frontend/")
    print("Continuing in demo mode...")

    # ============================================================================
    # EXEMPLO 1: Convert Qiskit Circuit to LightQOS
    # ============================================================================

print("=" * 80)
print("EXAMPLE 1: Qiskit → LightQOS → Execution with EFAL/EMF/TLM")
print("=" * 80)

# Create circuit in Qiskit (default)
qiskit_circ = QiskitCircuit(3, 3)
qiskit_circ.h(0)
qiskit_circ.cx(0, 1)
qiskit_circ.cx(1, 2)
qiskit_circ.measure([0, 1, 2], [0, 1, 2])

print("\n📊 Original Qiskit Circuit:")
print(qiskit_circ)

# Convert to LightQOS
try:
    lightqos_circ = qiskit_to_lightqos(qiskit_circ)

    print("\n✅ Converted to LightQOS successfully!")
    print(f"   - Qubits: {lightqos_circ.num_qubits}")
    print(f"   - Operações: {len(lightqos_circ.operations)}")

    # Add temporal contracts (unique LightQOS feature)
    strict_sla = TemporalContract.strict()
    lightqos_circ.add_temporal_contract(strict_sla)
    print(f"   - Contrato Temporal: {strict_sla.type}")

    # Executar com LightQOS features (EFAL/EMF/TLM)
    print("\n🚀 Executing with LightQOS (using EFAL for optimization)...")
    results = lightqos_circ.execute(
        backend="simulator",
        shots=1024,
        use_efal=True,  # Enables field optimization
        use_emf=True,  # Uses the entanglement pool
        use_tlm=True,  # Applies temporal scheduling
    )

    print("\n📈 LightQOS Results:")
    print(f"   Counts: {results.get('counts')}")
    print(f"   Fidelidade EFAL: {results.get('efal_fidelity', 'N/A')}")
    print(f"   Tempo TLM: {results.get('tlm_execution_time', 'N/A')}ms")

except Exception as e:
    print(f"\n⚠️  Modo simulado (LightQOS não instalado): {e}")
    # Fallback: execute com Qiskit pure
    simulator = AerSimulator()
    compiled = transpile(qiskit_circ, simulator)
    job = simulator.run(compiled, shots=1024)
    result = job.result()
    print("\n📈 Resultados Qiskit (fallback):")
    print(f"   Counts: {result.get_counts()}")

    # ============================================================================
    # EXEMPLO 2: 3-qubit GHZ state com Temporal Contracts
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 2: GHZ State with Temporal Contracts")
print("=" * 80)

qiskit_ghz = QiskitCircuit(3, 3)
qiskit_ghz.h(0)
qiskit_ghz.cx(0, 1)
qiskit_ghz.cx(0, 2)
qiskit_ghz.measure_all()

print("\n📊 GHZ circuit in Qiskit:")
print(qiskit_ghz.draw("text"))

try:
    lightqos_ghz = qiskit_to_lightqos(qiskit_ghz)

    # Apply contracts temporal diferentes
    print("\n⚙️  Applying contracts temporal...")

    # Gate H: precisa de high coherence
    lightqos_ghz.set_gate_contract(0, TemporalContract.strict())

    # CNOTs: podin ter margin flexível
    lightqos_ghz.set_gate_contract(1, TemporalContract.flexible(tolerance_ns=10))
    lightqos_ghz.set_gate_contract(2, TemporalContract.flexible(tolerance_ns=10))

    print("   ✅ H gate: Strict contract (high prioridade)")
    print("   ✅ CNOT gates: Flexible contract (±10ns tolerância)")

    # Executar
    results = lightqos_ghz.execute(backend="simulator", shots=2048)

    print("\n📈 Results:")
    counts = results.get("counts", {})
    print(f"   |000⟩: {counts.get('000', 0)} ({counts.get('000', 0) / 2048 * 100:.1f}%)")
    print(f"   |111⟩: {counts.get('111', 0)} ({counts.get('111', 0) / 2048 * 100:.1f}%)")
    print("   Fidelidade esperada: >95% (GHZ ideal)")

except Exception as e:
    print(f"\n⚠️  Modo simulado: {e}")

    # ============================================================================
    # EXEMPLO 3: LightQOS → Qiskit (ida e volta)
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 3: LightQOS → Qiskit (reverse conversion)")
print("=" * 80)

try:
    # Create circuit nativo in LightQOS
    lightqos_native = LightQOSCircuit(2, name="bell_pair_lightqos")
    lightqos_native.h(0)
    lightqos_native.cx(0, 1)
    lightqos_native.measure([0, 1])

    print("\n📊 Native LightQOS circuit:")
    print(lightqos_native.to_qasm3())

    # Converter de volta to Qiskit
    qiskit_from_lightqos = lightqos_to_qiskit(lightqos_native)

    print("\n✅ Converted back to Qiskit!")
    print(qiskit_from_lightqos)

    # Executar in simulador Qiskit
    simulator = AerSimulator()
    compiled = transpile(qiskit_from_lightqos, simulator)
    job = simulator.run(compiled, shots=1024)
    result = job.result()

    print("\n📈 Results (executed in Qiskit Aer):")
    print(result.get_counts())

except Exception as e:
    print(f"\n⚠️  Modo simulado: {e}")

    # ============================================================================
    # EXEMPLO 4: Comparison de Performance: Qiskit vs LightQOS
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 4: Benchmark - Qiskit vs LightQOS")
print("=" * 80)


# Create circuit de test (10 qubits, 50 gates)
test_circ = QiskitCircuit(10)
for i in range(9):
    test_circ.h(i)
    test_circ.cx(i, i + 1)
for i in range(10):
    test_circ.rz(0.5, i)
test_circ.measure_all()

print(f"\n📊 Circuito de teste: {test_circ.num_qubits} qubits, {test_circ.size()} portas")

try:
    # Benchmark 1: Qiskit pure
    print("\n⏱️  Benchmark 1: Qiskit (transpilação padrão)")
    start = time.time()
    simulator = AerSimulator()
    compiled_qiskit = transpile(test_circ, simulator, optimization_level=3)
    job = simulator.run(compiled_qiskit, shots=100)
    result = job.result()
    qiskit_time = time.time() - start
    print(f"   Tempo: {qiskit_time * 1000:.2f}ms")
    print(f"   Portas após transpilação: {compiled_qiskit.size()}")

    # Benchmark 2: LightQOS com EFAL
    print("\n⏱️  Benchmark 2: LightQOS (com EFAL + TLM)")
    start = time.time()
    lightqos_test = qiskit_to_lightqos(test_circ)
    results = lightqos_test.execute(
        backend="simulator", shots=100, use_efal=True, use_tlm=True, optimize=True
    )
    lightqos_time = time.time() - start
    print(f"   Tempo: {lightqos_time * 1000:.2f}ms")
    print(f"   Otimização EFAL: {results.get('efal_optimization', 'N/A')}")

    # Comparison
    print("\n📊 Comparison:")
    if lightqos_time < qiskit_time:
        speedup = qiskit_time / lightqos_time
        print(f"   🚀 LightQOS {speedup:.2f}x mais rápido!")
    else:
        print(f"   ⚖️  Qiskit {qiskit_time / lightqos_time:.2f}x mais rápido")

except Exception as e:
    print(f"\n⚠️  Benchmark não disponível: {e}")

    # ============================================================================
    # EXEMPLO 5: Hardware Real - IBM Quantum
    # ============================================================================

print("\n" + "=" * 80)
print("EXEMPLO 5: Execution in Hardware Real IBM Quantum")
print("=" * 80)

print("\n⚙️  Paira executar in hardware real, você precisa:")
print("   1. Conta IBM Quantum: https://quantum.ibm.com/")
print("   2. Token de API configurado")
print("   3. Descomentar o código abaixo")

"""
# Descomente for execute in hardware real:

from qiskit_ibm_runtime import QiskitRuntimeService

# Configurar serviço IBM
service = QiskitRuntimeService(
    channel="ibm_quantum",
    token="SEU_TOKEN_AQUI"
)

# Escolher backend
backend = service.backend("ibm_brisbane")  # ou outro available
print(f"Backend selecionado: {backend.name}")
print(f"Qubits: {backend.num_qubits}")

# Create circuit simples
ibm_circ = QiskitCircuit(5)
ibm_circ.h(0)
for i in range(4):
    ibm_circ.cx(i, i+1)
ibm_circ.measure_all()

# Convert to LightQOS
lightqos_ibm = qiskit_to_lightqos(ibm_circ)

# Add contracts de fidelity
lightqos_ibm.add_fidelity_contract(min_fidelity=0.95)

# Executar
results = lightqos_ibm.execute(
    backend=backend,
    shots=1024,
    use_efal=True,  # Optimization de topologia IBM
    use_error_mitigation=True
)

print(f"Job ID: {results.job_id}")
print(f"Results: {results.counts}")
print(f"Achieved fidelity: {results.fidelity}")
"""

print("\n✅ Example complete! LightQOS + Qiskit integrado com success.")
print("\n📚 Next steps:")
print("   - Ver: bell_state_with_sla.py (States de Bell com SLA)")
print("   - Ver: high_dimensional_ghz.py (GHZ in 37 dimensions)")
print("   - Ver: ../docs/tutorials/ (documentation complete)")
