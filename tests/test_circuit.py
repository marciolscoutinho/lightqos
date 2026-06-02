# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# test_circuit.py — Circuit Unit Tests — Python SDK without Rust dependency
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 10-02-2025
# All rights reserved.
# -----------------------------------------------------------------------------

"""
tests/test_circuit.py
=====================
Unit Tests — Python SDK (without Rust kernel dependency)

Tests: QuantumCircuit, TemporalContract, integrations, basic examples.
All tests run without the compiled Rust module.
"""

import pytest
import math
import sys
import os

# Add frontend to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), '..', 'frontend'))

try:
    from lightqos.circuit import QuantumCircuit
    from lightqos.contracts import TemporalContract
    SDK_AVAILABLE = True
except ImportError:
    SDK_AVAILABLE = False

pytestmark = pytest.mark.skipif(not SDK_AVAILABLE, reason="lightqos SDK not installed")


# =============================================================================
# TemporalContract
# =============================================================================

class TestTemporalContract:
    """Tests for the temporal contract system."""

    def test_creation_defaults(self):
        c = TemporalContract(operation="CNOT", deadline_ms=100.0)
        assert c.operation == "CNOT"
        assert c.deadline_ms == 100.0
        assert c.priority == 5
        assert not c.fulfilled

    def test_priority_clamping(self):
        c_low  = TemporalContract("X", 50.0, priority=0)
        c_high = TemporalContract("X", 50.0, priority=99)
        assert c_low.priority >= 1
        assert c_high.priority <= 10

    def test_id_is_uuid(self):
        c = TemporalContract("H", 100.0)
        assert len(c.id) == 36
        assert c.id.count('-') == 4

    def test_two_contracts_have_different_ids(self):
        c1 = TemporalContract("H", 100.0)
        c2 = TemporalContract("H", 100.0)
        assert c1.id != c2.id

    def test_time_remaining_positive_initially(self):
        c = TemporalContract("X", deadline_ms=10_000.0)
        assert c.time_remaining_ms() > 0

    def test_not_expired_immediately(self):
        c = TemporalContract("SWAP", deadline_ms=5_000.0)
        assert not c.is_expired()

    def test_repr_contains_operation(self):
        c = TemporalContract("CZ_GATE", 200.0, priority=8)
        r = repr(c)
        assert "CZ_GATE" in r

    def test_fulfill(self):
        c = TemporalContract("CCX", 500.0)
        assert not c.fulfilled
        c.fulfilled = True
        assert c.fulfilled


# =============================================================================
# QuantumCircuit — creation and structure
# =============================================================================

class TestQuantumCircuitCreation:
    """Creation and basic structure tests."""

    def test_single_qubit(self):
        qc = QuantumCircuit(1)
        assert qc.n_qubits == 1

    def test_multi_qubit(self):
        qc = QuantumCircuit(5)
        assert qc.n_qubits == 5

    def test_name(self):
        qc = QuantumCircuit(2, name="Bell")
        assert qc.name == "Bell"

    def test_default_name(self):
        qc = QuantumCircuit(3)
        assert qc.name is not None

    def test_empty_circuit_has_zero_gates(self):
        qc = QuantumCircuit(2)
        assert qc.num_gates() == 0

    def test_empty_circuit_has_zero_depth(self):
        qc = QuantumCircuit(2)
        assert qc.depth() == 0

    def test_invalid_qubit_count_raises(self):
        with pytest.raises((ValueError, AssertionError)):
            QuantumCircuit(0)

    def test_copy(self):
        qc = QuantumCircuit(2)
        qc.h(0)
        qc_copy = qc.copy()
        assert qc_copy.n_qubits == qc.n_qubits
        assert qc_copy.num_gates() == qc.num_gates()
        # Modifying the copy does not affect the original
        qc_copy.x(1)
        assert qc.num_gates() != qc_copy.num_gates()


# =============================================================================
# QuantumCircuit — 1-qubit gates
# =============================================================================

class TestSingleQubitGates:
    """1-qubit gate tests."""

    def test_h_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.h(0)
        assert qc.num_gates() == 1

    def test_x_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.x(0)
        assert qc.num_gates() == 1

    def test_y_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.y(0)
        assert qc.num_gates() == 1

    def test_z_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.z(0)
        assert qc.num_gates() == 1

    def test_s_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.s(0)
        assert qc.num_gates() == 1

    def test_t_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.t(0)
        assert qc.num_gates() == 1

    def test_sdg_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.sdg(0)
        assert qc.num_gates() == 1

    def test_rx_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.rx(math.pi / 2, 0)
        assert qc.num_gates() == 1

    def test_ry_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.ry(math.pi / 4, 0)
        assert qc.num_gates() == 1

    def test_rz_adds_gate(self):
        qc = QuantumCircuit(1)
        qc.rz(math.pi / 8, 0)
        assert qc.num_gates() == 1

    def test_multiple_gates_same_qubit(self):
        qc = QuantumCircuit(1)
        qc.h(0)
        qc.x(0)
        qc.h(0)
        assert qc.num_gates() == 3

    def test_out_of_range_qubit_raises(self):
        qc = QuantumCircuit(2)
        with pytest.raises((ValueError, IndexError, AssertionError)):
            qc.h(5)  # qubit 5 does not exist

    def test_chaining_returns_circuit(self):
        """Gates should be chainable."""
        qc = QuantumCircuit(1)
        result = qc.h(0)
        # It may return self or None — both are acceptable
        assert result is None or result is qc


# =============================================================================
# QuantumCircuit — 2-qubit gates
# =============================================================================

class TestTwoQubitGates:
    """2-qubit gate tests."""

    def test_cnot_adds_gate(self):
        qc = QuantumCircuit(2)
        qc.cnot(0, 1)
        assert qc.num_gates() == 1

    def test_cx_alias(self):
        qc = QuantumCircuit(2)
        qc.cx(0, 1)
        assert qc.num_gates() == 1

    def test_cz_adds_gate(self):
        qc = QuantumCircuit(2)
        qc.cz(0, 1)
        assert qc.num_gates() == 1

    def test_swap_adds_gate(self):
        qc = QuantumCircuit(2)
        qc.swap(0, 1)
        assert qc.num_gates() == 1

    def test_cp_adds_gate(self):
        qc = QuantumCircuit(2)
        qc.cp(math.pi / 4, 0, 1)
        assert qc.num_gates() == 1

    def test_same_qubit_control_target_raises(self):
        qc = QuantumCircuit(2)
        with pytest.raises((ValueError, AssertionError)):
            qc.cnot(0, 0)

    def test_count_2q_gates(self):
        qc = QuantumCircuit(3)
        qc.h(0)         # 1Q
        qc.cnot(0, 1)   # 2Q
        qc.x(2)         # 1Q
        qc.cz(1, 2)     # 2Q
        assert qc.count_2q_gates() == 2


# =============================================================================
# QuantumCircuit — Bell State
# =============================================================================

class TestBellState:
    """Specific tests for Bell states."""

    def test_bell_phi_plus_structure(self):
        qc = QuantumCircuit(2)
        qc.h(0)
        qc.cnot(0, 1)
        assert qc.num_gates() == 2
        assert qc.n_qubits == 2

    def test_bell_phi_plus_execution(self):
        qc = QuantumCircuit(2)
        qc.h(0)
        qc.cnot(0, 1)
        qc.measure([0, 1])
        result = qc.execute(backend="simulator", shots=1024)
        # Should contain only |00⟩ and |11⟩
        assert set(result.counts.keys()).issubset({"00", "11"})
        # Each one with ~50%
        total = sum(result.counts.values())
        for count in result.counts.values():
            assert 0.35 < count / total < 0.65

    def test_all_four_bell_states_execute(self):
        configs = [
            ([], []),           # |Φ+⟩
            ([('z', 0)], []),   # |Φ-⟩
            ([('x', 1)], []),   # |Ψ+⟩
            ([('x', 1), ('z', 0)], []),  # |Ψ-⟩
        ]
        for pre_gates, _ in configs:
            qc = QuantumCircuit(2)
            for gate, qubit in pre_gates:
                getattr(qc, gate)(qubit)
            qc.h(0)
            qc.cnot(0, 1)
            qc.measure([0, 1])
            result = qc.execute(backend="simulator", shots=256)
            assert len(result.counts) > 0

    def test_ghz_state(self):
        qc = QuantumCircuit(3)
        qc.h(0)
        qc.cnot(0, 1)
        qc.cnot(0, 2)
        qc.measure([0, 1, 2])
        result = qc.execute(backend="simulator", shots=1024)
        # GHZ: only |000⟩ and |111⟩
        assert set(result.counts.keys()).issubset({"000", "111"})


# =============================================================================
# QuantumCircuit — measurements and execution
# =============================================================================

class TestExecution:
    """Circuit execution tests."""

    def test_measure_specific_qubits(self):
        qc = QuantumCircuit(3)
        qc.h(0)
        qc.measure([0, 1])  # measure only 0 and 1
        result = qc.execute(backend="simulator", shots=256)
        assert result is not None

    def test_measure_all_shorthand(self):
        qc = QuantumCircuit(2)
        qc.x(0)
        qc.measure_all()
        result = qc.execute(backend="simulator", shots=256)
        # X on qubit 0 → should produce |10⟩ (or |01⟩ depending on bit order)
        assert len(result.counts) >= 1

    def test_shots_count(self):
        qc = QuantumCircuit(1)
        qc.h(0)
        qc.measure([0])
        result = qc.execute(backend="simulator", shots=512)
        total = sum(result.counts.values())
        assert total == 512

    def test_deterministic_x_gate(self):
        qc = QuantumCircuit(1)
        qc.x(0)
        qc.measure([0])
        result = qc.execute(backend="simulator", shots=128)
        # X|0⟩ = |1⟩ → should always measure 1
        assert result.counts.get("1", 0) == 128

    def test_result_has_counts(self):
        qc = QuantumCircuit(1)
        qc.measure([0])
        result = qc.execute(backend="simulator", shots=64)
        assert hasattr(result, "counts")
        assert isinstance(result.counts, dict)

    def test_temporal_contract_with_execution(self):
        qc = QuantumCircuit(1)
        qc.h(0)
        qc.measure([0])
        contract = TemporalContract("H_GATE", deadline_ms=10_000.0)
        result = qc.execute(backend="simulator", shots=128, contract=contract)
        assert result is not None

    def test_circuit_depth_bell(self):
        qc = QuantumCircuit(2)
        qc.h(0)
        qc.cnot(0, 1)
        # Depth: H(0) in parallel → CNOT → depth = 2
        assert qc.depth() >= 2

    def test_inverse_circuit(self):
        qc = QuantumCircuit(1)
        qc.h(0)
        qc.s(0)
        inv = qc.inverse()
        assert inv.num_gates() == qc.num_gates()

    def test_compose_circuits(self):
        qc1 = QuantumCircuit(2)
        qc1.h(0)

        qc2 = QuantumCircuit(2)
        qc2.cnot(0, 1)

        qc1.compose(qc2)
        assert qc1.num_gates() == 2


# =============================================================================
# Grover algorithm (integration)
# =============================================================================

class TestGrover:
    """Integration test — 2-qubit Grover."""

    def test_grover_2q_finds_target(self):
        """2-qubit Grover with target |11⟩."""
        target = 3  # |11⟩

        # 2Q Grover circuit — 1 iteration (optimal for N=4)
        qc = QuantumCircuit(2)

        # Superposition
        qc.h(0); qc.h(1)

        # Oracle: marks |11⟩ with phase -1 (CZ = phase -1 on |11⟩)
        qc.cz(0, 1)

        # Diffuser
        qc.h(0); qc.h(1)
        qc.x(0); qc.x(1)
        qc.cz(0, 1)
        qc.x(0); qc.x(1)
        qc.h(0); qc.h(1)

        qc.measure([0, 1])
        result = qc.execute(backend="simulator", shots=1024)

        target_str = format(target, "02b")
        target_count = result.counts.get(target_str, 0)
        prob = target_count / 1024

        assert prob > 0.8, f"Grover failed: P(|11⟩) = {prob:.2%}"


# =============================================================================
# QuantumCircuit — advanced properties
# =============================================================================

class TestCircuitProperties:
    """Advanced circuit property tests."""

    def test_num_qubits_property(self):
        for n in [1, 2, 4, 8, 16]:
            qc = QuantumCircuit(n)
            assert qc.n_qubits == n

    def test_barrier_does_not_count_as_gate(self):
        qc = QuantumCircuit(2)
        qc.h(0)
        if hasattr(qc, 'barrier'):
            qc.barrier()
        qc.x(1)
        assert qc.num_gates() == 2

    def test_circuit_to_dict_or_repr(self):
        qc = QuantumCircuit(2, name="TestCircuit")
        qc.h(0)
        qc.cnot(0, 1)
        # Should have some string representation
        r = str(qc) or repr(qc)
        assert len(r) > 0
