#!/usr/bin/env python3
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# bell_state.py — Bell State Example — all four Bell states with analysis
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 19-06-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
examples/bell_state.py
======================
Basic example: Bell State with LightQOS

Demonstrates:
- Quantum circuit creation
- H and CNOT gates
- Simulator execution
- Result analysis with Shadow Tomography
- Use of temporal contracts (TLM)
"""

from lightqos import QuantumCircuit, TemporalContract
import math


def bell_state_phi_plus() -> QuantumCircuit:
    """Creates the circuit for the Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2"""
    circuit = QuantumCircuit(2, name="Bell |Φ+⟩")
    circuit.h(0)  # Hadamard on qubit 0 → superposition
    circuit.cnot(0, 1)  # CNOT → entanglement
    return circuit


def bell_state_phi_minus() -> QuantumCircuit:
    """Creates |Φ-⟩ = (|00⟩ - |11⟩)/√2"""
    circuit = QuantumCircuit(2, name="Bell |Φ-⟩")
    circuit.h(0)
    circuit.z(0)  # Phase -1 on qubit 0
    circuit.cnot(0, 1)
    return circuit


def bell_state_psi_plus() -> QuantumCircuit:
    """Creates |Ψ+⟩ = (|01⟩ + |10⟩)/√2"""
    circuit = QuantumCircuit(2, name="Bell |Ψ+⟩")
    circuit.h(0)
    circuit.x(1)  # Flip qubit 1
    circuit.cnot(0, 1)
    return circuit


def bell_state_psi_minus() -> QuantumCircuit:
    """Creates |Ψ-⟩ = (|01⟩ - |10⟩)/√2 — spin singlet"""
    circuit = QuantumCircuit(2, name="Bell |Ψ-⟩")
    circuit.h(0)
    circuit.x(1)
    circuit.z(0)
    circuit.cnot(0, 1)
    return circuit


def analyse_bell_state(circuit: QuantumCircuit, shots: int = 2048, use_shadow: bool = False):
    """
    Executes and analyzes a Bell state.

    Args:
        circuit:    Bell-state circuit
        shots:      Number of executions
        use_shadow: Enable Shadow Tomography to estimate fidelity
    """
    print(f"\n{'=' * 50}")
    print(f"Bell State: {circuit.name}")
    print(f"{'=' * 50}")
    print(f"Profundidade: {circuit.depth()} | Gates: {circuit.num_gates()}")

    # Contract temporal (deadline 500ms)
    contract = TemporalContract(operation=f"bell_{circuit.name}", deadline_ms=500.0, priority=7)

    # Measure
    circuit.measure([0, 1])
    result = circuit.execute(
        backend="simulator",
        shots=shots,
        contract=contract,
        shadow_tomography=use_shadow,
        n_shadows=500 if use_shadow else 0,
    )

    # Contagens
    print(f"\nResultados ({shots} shots):")
    sorted_counts = sorted(result.counts.items(), key=lambda x: -x[1])
    for state, count in sorted_counts:
        bar = "█" * round(count / shots * 40)
        print(f"  |{state}⟩  {count:5d}  ({100 * count / shots:5.1f}%)  {bar}")

        # Verify expected distribution
    total = sum(result.counts.values())
    dominant = [s for s, c in result.counts.items() if c / total > 0.40]
    if len(dominant) == 2:
        print(f"\n✅ Bell state confirmado: {dominant[0]} e {dominant[1]} com ~50% cada")
    else:
        print(f"\n⚠️  Distribuição inesperada: {result.counts}")

        # Shadow Tomography
    if use_shadow and hasattr(result, "shadow_result"):
        sr = result.shadow_result
        print("\nShadow Tomography:")
        print(f"  Sombras recolhidas: {sr['num_shadows']}")
        print(f"  Fidelidade estimada: {sr['mean_fidelity']:.4f}")
        print(f"  Certificado estatístico: {sr['statistical_certificate']:.4f}")
        print(f"  Qualidade: {sr['reconstruction_quality']}")

        # Contract
    if contract.fulfilled:
        print(f"\nContrato TLM: ✅ cumprido em {contract.elapsed_ms():.1f}ms")
    else:
        print(
            f"\nContrato TLM: ⚠️  expirado ({contract.elapsed_ms():.1f}ms > {contract.deadline_ms}ms)"
        )

    return result


def demo_entanglement_correlation():
    """Demonstrates correlation quantum de um Bell state."""
    print("\n📊 Quantum correlation demonstration")
    print("─" * 45)
    print("For |Φ+⟩ = (|00⟩ + |11⟩)/√2:")
    print("  • Measuring qubit 0 as |0⟩ → qubit 1 collapses to |0⟩")
    print("  • Measuring qubit 0 as |1⟩ → qubit 1 collapses to |1⟩")
    print("  • Perfect correlation: CHSH = 2√2 ≈ 2.828 > 2 (threshold classical)")

    chsh = 2 * math.sqrt(2)
    print(f"\n  Valor CHSH quântico: {chsh:.4f}")
    print("  Limite clássico Bell: 2.0000")
    print(f"  Violação de Bell: {'✅ Yes' if chsh > 2 else '❌ No'} ({chsh:.4f} > 2)")


def main():
    print("🌟 LightQOS — Demonstration de Bell States")
    print("==========================================\n")

    shots = 2048

    # All os 4 Bell states
    circuits = [
        bell_state_phi_plus(),
        bell_state_phi_minus(),
        bell_state_psi_plus(),
        bell_state_psi_minus(),
    ]

    for circuit in circuits:
        analyse_bell_state(circuit, shots=shots, use_shadow=False)

        # Analysis com Shadow Tomography on the |Φ+⟩
    print("\n\n🔬 Advanced analysis with Shadow Tomography")
    phi_plus = bell_state_phi_plus()
    analyse_bell_state(phi_plus, shots=shots, use_shadow=True)

    # Demonstration de correlation
    demo_entanglement_correlation()

    print("\n\n✅ Demonstration completed!")


if __name__ == "__main__":
    main()
