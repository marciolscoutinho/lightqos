# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# bell_state_with_sla.py — Bell State with SLA — temporal contract integration demo
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 22-06-2024
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Example: Bell State with Service Level Agreements (SLA)
Demonstrates the LightQOS temporal and fidelity contract system
"""

try:
    from lightqos import QuantumCircuit, TemporalContract, FidelityContract

    LIGHTQOS_AVAILABLE = True
except ImportError:
    print("⚠️  LightQOS not installed. Demo mode enabled.")
    LIGHTQOS_AVAILABLE = False

import numpy as np


# ============================================================================
# PARTE 1: State de Bell Básico com Contract Temporal Strict
# ============================================================================

print("=" * 80)
print("EXAMPLE 1: Bell State with STRICT Temporal Contract")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    # Create circuit
    circuit = QuantumCircuit(2, name="bell_strict")

    # Apply gates com strict contract (maximum priority temporal)
    strict_contract = TemporalContract.strict()

    circuit.h(0, contract=strict_contract)
    circuit.cx(0, 1, contract=strict_contract)
    circuit.measure([0, 1], contract=strict_contract)

    print("\n📊 Circuit created:")
    print("   - 2 qubits")
    print(f"   - Contrato: {strict_contract.type}")
    print(f"   - Janela temporal: {strict_contract.max_duration_ns}ns")
    print(f"   - Prioridade TLM: {strict_contract.priority}")

    # Executar
    print("\n🚀 Executing with TLM (Temporal Layer Manager)...")
    results = circuit.execute(backend="simulator", shots=2048, use_tlm=True, verbose=True)

    # Analysis
    print("\n📈 Results:")
    counts = results.get("counts", {})

    bell_fidelity = (counts.get("00", 0) + counts.get("11", 0)) / 2048
    print(f"   |00⟩: {counts.get('00', 0)} ({counts.get('00', 0) / 2048 * 100:.1f}%)")
    print(f"   |11⟩: {counts.get('11', 0)} ({counts.get('11', 0) / 2048 * 100:.1f}%)")
    print(f"   Fidelidade Bell: {bell_fidelity * 100:.2f}%")

    # TLM information
    tlm_data = results.get("tlm_report", {})
    print("\n⏱️  Relatório TLM:")
    print(f"   Tempo total execução: {tlm_data.get('total_time_ns', 'N/A')}ns")
    print(f"   Contratos cumpridos: {tlm_data.get('contracts_met', 'N/A')}/3")
    print(f"   Violações: {tlm_data.get('violations', 0)}")

else:
    print("\n⚠️  Demo mode - install LightQOS to run")

    # ============================================================================
    # PARTE 2: Contract Comparison: Strict vs Flexible vs Relaxed
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 2: Temporal Contract Comparison")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    contracts = {
        "strict": TemporalContract.strict(),
        "flexible": TemporalContract.flexible(tolerance_ns=50),
        "relaxed": TemporalContract.relaxed(tolerance_ns=200),
    }

    results_comparison = {}

    for contract_name, contract in contracts.items():
        print(f"\n⚙️  Testando contrato: {contract_name.upper()}")

        # Create circuit
        circ = QuantumCircuit(2, name=f"bell_{contract_name}")
        circ.h(0, contract=contract)
        circ.cx(0, 1, contract=contract)
        circ.measure([0, 1])

        # Executar
        res = circ.execute(backend="simulator", shots=1024, use_tlm=True)

        # Guardar
        results_comparison[contract_name] = res

        # Mostrar
        tlm = res.get("tlm_report", {})
        print(f"   Tempo: {tlm.get('total_time_ns', 0)}ns")
        print(f"   Fidelidade: {tlm.get('fidelity', 0) * 100:.2f}%")
        print(f"   Violações: {tlm.get('violations', 0)}")

        # Analysis comparativa
    print("\n📊 Comparative Analysis:")
    print(f"{'Contract':<12} {'Time (ns)':<12} {'Fidelity':<12} {'Violations':<12}")
    print("-" * 50)

    for name, res in results_comparison.items():
        tlm = res.get("tlm_report", {})
        print(
            f"{name:<12} {tlm.get('total_time_ns', 0):<12} "
            f"{tlm.get('fidelity', 0) * 100:<12.2f} {tlm.get('violations', 0):<12}"
        )

    print("\n💡 Conclusion:")
    print("   - STRICT: Higher fidelity, slower, on the violations")
    print("   - FLEXIBLE: Balanced, good fidelity, few violations")
    print("   - RELAXED: Faster, lower fidelity, possible violations")

else:
    print("\n⚠️  Demo mode")

    # ============================================================================
    # PARTE 3: Contract de Fidelity com Auto-retry
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 3: Fidelity Contract with Auto-retry")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    # Create circuit com contract de fidelity
    circuit_fid = QuantumCircuit(2, name="bell_fidelity_contract")

    # Definir contract: minimum 95% fidelity
    fidelity_contract = FidelityContract(min_fidelity=0.95, max_retries=5, auto_optimize=True)

    print("\n📋 Contract de Fidelity:")
    print(f"   - Fidelidade mínima: {fidelity_contract.min_fidelity * 100}%")
    print(f"   - Tentativas máximas: {fidelity_contract.max_retries}")
    print(f"   - Otimização automática: {fidelity_contract.auto_optimize}")

    # Build circuit
    circuit_fid.h(0)
    circuit_fid.cx(0, 1)
    circuit_fid.measure([0, 1])

    # Apply contract
    circuit_fid.add_fidelity_contract(fidelity_contract)

    # Executar
    print("\n🚀 Executing with the fidelity contract...")
    print("   (the systin will retry until it reaches ≥95% fidelity)")

    results_fid = circuit_fid.execute(
        backend="simulator",
        shots=4096,
        use_efal=True,  # Usar EFAL para optimization
        use_error_mitigation=True,
    )

    # Analysis
    print("\n📈 Results:")
    fid_report = results_fid.get("fidelity_report", {})
    print(f"   Tentativas realizadas: {fid_report.get('attempts', 'N/A')}")
    print(f"   Fidelidade alcançada: {fid_report.get('final_fidelity', 0) * 100:.2f}%")
    print(f"   Contrato cumprido: {'✅ YES' if fid_report.get('contract_met') else '❌ NO'}")

    if fid_report.get("optimizations_applied"):
        print("\n⚙️  Otimizações aplicadas pelo EFAL:")
        for opt in fid_report["optimizations_applied"]:
            print(f"   - {opt}")

else:
    print("\n⚠️  Demo mode")

    # ============================================================================
    # PARTE 4: State de Bell com HIO (Holographic I/O)
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 4: Bell State with Holographic Measurement (HIO)")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    # Create circuit
    circuit_hio = QuantumCircuit(2, name="bell_holographic")
    circuit_hio.h(0)
    circuit_hio.cx(0, 1)

    # Holographic measurement (does not fully collapse)
    circuit_hio.measure([0, 1], holographic=True, bases=["Z", "X", "Y"])

    print("\n📊 Circuit with HIO:")
    print("   - Measurement in 3 bases: Z, X, Y")
    print("   - Shadow copies: 50 amostras")
    print("   - Statistical guarantee: 95% confiança")

    # Executar
    print("\n🚀 Executing with HIO (Holographic I/O)...")
    results_hio = circuit_hio.execute(
        backend="simulator",
        shots=100,  # Menos shots necessários com HIO
        use_hio=True,
    )

    # Dados HIO
    hio_data = results_hio.get("hio_data", {})

    print("\n📈 Results HIO:")
    print(f"   Confiança: {hio_data.get('confidence', 0) * 100:.1f}%")
    print(f"   Fidelidade estimada: {hio_data.get('fidelity', 0) * 100:.2f}%")
    print(f"   Concorrência: {hio_data.get('concurrence', 0):.4f}")

    # Observable views
    print("\n🔭 Observable Views:")
    views = hio_data.get("observable_views", {})
    for basis, data in views.items():
        print(f"   Base {basis}: ⟨{data.get('operator', '?')}⟩ = {data.get('expectation', 0):.4f}")

        # Shadow copies
    print("\n👥 Shadow Copies:")
    shadows = hio_data.get("shadow_copies", [])
    print(f"   Total: {len(shadows)} amostras")
    if shadows:
        print(f"   Exemplo: {shadows[0]}")

        # Matriz densidade reconstruída
    if "density_matrix" in hio_data:
        rho = np.array(hio_data["density_matrix"])
        print("\n🎲 Matriz Densidade Reconstruída:")
        print(f"   Shape: {rho.shape}")
        print(f"   Traço: {np.trace(rho):.4f} (deve ser ~1)")
        print(f"   Pureza: {np.trace(rho @ rho):.4f}")

else:
    print("\n⚠️  Demo mode")

    # ============================================================================
    # PARTE 5: Multi-Bell States com EMF (Entangled Memory Fabric)
    # ============================================================================

print("\n" + "=" * 80)
print("EXAMPLE 5: Multiple Bell States with EMF")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    print("\n⚙️  Creating 4 Bell pairs with EMF...")

    # Create circuit de 8 qubits (4 pairs)
    circuit_multi = QuantumCircuit(8, name="multi_bell_emf")

    # Enable EMF (Entangled Memory Fabric)
    circuit_multi.enable_emf(pool_size=10)

    # Create 4 pairs de Bell
    pairs = [(0, 1), (2, 3), (4, 5), (6, 7)]

    for i, (q1, q2) in enumerate(pairs):
        print(f"   Par {i + 1}: qubits {q1}-{q2}")
        circuit_multi.h(q1)
        circuit_multi.cx(q1, q2)

        # Measurement
    circuit_multi.measure(list(range(8)))

    # Executar
    print("\n🚀 Executing com EMF...")
    results_multi = circuit_multi.execute(backend="simulator", shots=1024, use_emf=True)

    # Analysis EMF
    emf_data = results_multi.get("emf_report", {})
    print("\n📊 Relatório EMF:")
    print(f"   Pares criados: {emf_data.get('pairs_created', 0)}")
    print(f"   Pares reutilizados do pool: {emf_data.get('pairs_reused', 0)}")
    print(f"   Fidelidade média: {emf_data.get('avg_fidelity', 0) * 100:.2f}%")
    print(f"   Ergotropia total: {emf_data.get('total_ergotropy', 0):.4f}")

    # Verificar correlações
    counts = results_multi.get("counts", {})
    perfect_correlations = sum(
        1
        for state, count in counts.items()
        if state[0] == state[1]
        and state[2] == state[3]
        and state[4] == state[5]
        and state[6] == state[7]
    )

    print("\n🔗 Correlações de Bell:")
    print(f"   Estados perfeitamente correlacionados: {perfect_correlations}")
    print(f"   % de correlação: {perfect_correlations / len(counts) * 100:.1f}%")

else:
    print("\n⚠️  Demo mode")

    # ============================================================================
    # PARTE 6: Bell State com Rollback (Snapshot TLM)
    # ============================================================================

print("\n" + "=" * 80)
print("EXEMPLO 6: Bell State com Rollback (TLM Snapshot)")
print("=" * 80)

if LIGHTQOS_AVAILABLE:
    # Create circuit com checkpoints
    circuit_snap = QuantumCircuit(2, name="bell_snapshot")

    # Checkpoint 1: State inicial
    circuit_snap.snapshot("initial")

    # Create Bell
    circuit_snap.h(0)
    circuit_snap.snapshot("after_hadamard")

    circuit_snap.cx(0, 1)
    circuit_snap.snapshot("bell_state")

    # Measurement
    circuit_snap.measure([0, 1])

    print("\n📸 Snapshots definidos:")
    print("   1. initial (antes de qualquer gate)")
    print("   2. after_hadamard (após H)")
    print("   3. bell_state (após CNOT)")

    # Executar com possibilidade de rollback
    print("\n🚀 Executing com TLM Snapshots...")

    results_snap = circuit_snap.execute(
        backend="simulator", shots=512, use_tlm=True, enable_rollback=True, max_retries=3
    )

    # TLM Report
    tlm_snap = results_snap.get("tlm_report", {})
    print("\n📈 Relatório TLM:")
    print(f"   Execuções bem-sucedidas: {tlm_snap.get('successful_runs', 0)}")
    print(f"   Rollbacks realizados: {tlm_snap.get('rollbacks', 0)}")

    if tlm_snap.get("rollback_history"):
        print("\n↩️  Histórico de Rollbacks:")
        for rb in tlm_snap["rollback_history"]:
            print(f"   - {rb['reason']} → voltou para '{rb['snapshot']}'")

else:
    print("\n⚠️  Demo mode")

    # ============================================================================
    # RESUMO FINAL
    # ============================================================================

print("\n" + "=" * 80)
print("RESUMO: Recursos LightQOS Demonstrados")
print("=" * 80)

print("""
✅ Temporal Contracts (TLM):
   - Strict, Flexible, Relaxed
   - Garantias de timing
   - Priorização de operações

✅ Contracts de Fidelity:
   - Auto-retry até atingir fidelity
   - Automatic optimization com EFAL
   - Mitigação de erros

✅ Holographic I/O (HIO):
   - Measurement in múltiplas bases
   - Shadow copies
   - Garantias statistics
   - Reconstrução de densidade

✅ Entangled Memory Fabric (EMF):
   - Pool de pairs entrelaçados
   - Reutilização de recursos
   - Métricas termodinâmicas

✅ Snapshots & Rollback (TLM):
   - Checkpoints de state
   - Recovery automática
   - Retry inteligente

📚 Next exemplos:
   - high_dimensional_ghz.py (GHZ in 37 dimensions)
   - quantum_network.py (Network quantum distribuída)
   - t_hqc_demo.py (Transmutação Hamiltoniana)
""")

print("\n✅ Example complete!")
