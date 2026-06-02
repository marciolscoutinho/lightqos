# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# test_pyo3_integration.py — PyO3 Integration Tests — Python ↔ Rust kernel binding validation
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 09-06-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Integration Tests - PyO3 Bindings

Tests Python ↔ Rust integration through PyO3
"""

import pytest
import time

# Try to import the Rust module
try:
    from lightqos._rust import (
        EMFManager,
        ContractManager,
        TemporalContract,
        ShadowCollector,
        get_kernel_info,
        benchmark_emf,
    )

    RUST_AVAILABLE = True
except ImportError:
    RUST_AVAILABLE = False
    pytestmark = pytest.mark.skip(reason="Rust module not available")


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust module required")
class TestEMFManager:
    """Tests for EMFManager (Rust)"""

    def test_creation(self):
        """Tests manager creation"""
        emf = EMFManager(max_pairs=100, recycling_threshold=0.5)

        assert emf.num_pairs() == 0
        assert emf.avg_fidelity() == 0.0

    def test_generate_pair(self):
        """Tests pair generation"""
        emf = EMFManager(100, 0.5)

        pair_id = emf.generate_pair(0, 1, 0.95)

        assert isinstance(pair_id, str)
        assert len(pair_id) > 0
        assert emf.num_pairs() == 1

    def test_generate_multiple_pairs(self):
        """Tests generation of multiple pairs"""
        emf = EMFManager(100, 0.5)

        for i in range(50):
            emf.generate_pair(i, i + 50, 0.95)

        assert emf.num_pairs() == 50
        assert abs(emf.avg_fidelity() - 0.95) < 0.01

    def test_capacity_limit(self):
        """Tests capacity limit"""
        emf = EMFManager(max_pairs=10, recycling_threshold=0.5)

        # Fill capacity
        for i in range(10):
            emf.generate_pair(i, i + 10, 0.95)

        # Trying to add more should fail
        with pytest.raises(RuntimeError, match="capacity exceeded"):
            emf.generate_pair(20, 21, 0.95)

    def test_get_pair(self):
        """Tests pair retrieval"""
        emf = EMFManager(100, 0.5)
        pair_id = emf.generate_pair(0, 1, 0.95)

        pair = emf.get_pair(pair_id)

        assert pair.qubit_a == 0
        assert pair.qubit_b == 1
        assert abs(pair.fidelity - 0.95) < 0.01

    def test_consume_pair(self):
        """Tests pair consumption"""
        emf = EMFManager(100, 0.5)
        pair_id = emf.generate_pair(0, 1, 0.95)

        emf.consume_pair(pair_id)

        assert emf.num_pairs() == 0

        # Trying to consume again should fail
        with pytest.raises(KeyError):
            emf.consume_pair(pair_id)

    def test_age_all_pairs(self):
        """Tests pair aging"""
        emf = EMFManager(100, 0.5)

        for i in range(10):
            emf.generate_pair(i, i + 10, 0.95)

        initial_fidelity = emf.avg_fidelity()

        # Age
        emf.age_all_pairs()

        aged_fidelity = emf.avg_fidelity()

        # Fidelity should have decreased
        assert aged_fidelity < initial_fidelity
        assert aged_fidelity > 0.9  # ~0.95 * 0.99

    def test_recycle(self):
        """Tests recycling"""
        emf = EMFManager(100, recycling_threshold=0.8)

        # Generate pairs with different fidelities
        for i in range(10):
            fidelity = 0.95 if i < 5 else 0.75  # Half with low fidelity
            emf.generate_pair(i, i + 10, fidelity)

        assert emf.num_pairs() == 10

        # Recycle
        recycled = emf.recycle()

        # Only pairs with F < 0.8 should be recycled
        assert recycled == 5
        assert emf.num_pairs() == 5

    def test_statistics(self):
        """Tests statistics"""
        emf = EMFManager(100, 0.5)

        for i in range(10):
            emf.generate_pair(i, i + 10, 0.95)

        stats = emf.get_statistics()

        assert stats["total_generated"] == 10
        assert stats["active_pairs"] == 10
        assert abs(stats["avg_fidelity"] - 0.95) < 0.01
        assert stats["capacity"] == 100
        assert abs(stats["utilization"] - 0.1) < 0.01

    def test_performance_vs_python(self):
        """Tests performance vs pure Python"""
        emf = EMFManager(1000, 0.5)

        # Benchmark Rust
        start = time.perf_counter()
        for i in range(1000):
            emf.generate_pair(i, i + 1000, 0.95)
        rust_time = time.perf_counter() - start

        # Rust should be significantly faster
        # (no direct comparison with Python here, but it can be added)
        assert rust_time < 0.1  # Should be < 100ms for 1000 operations


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust module required")
class TestContractManager:
    """Tests for ContractManager (Rust)"""

    def test_contract_creation_strict(self):
        """Tests strict contract creation"""
        contract = TemporalContract.strict()

        assert contract.max_duration_ns == 1_000_000
        assert contract.max_phase_error == 0.01
        assert contract.min_fidelity == 0.95

    def test_contract_creation_relaxed(self):
        """Tests relaxed contract creation"""
        contract = TemporalContract.relaxed()

        assert contract.max_duration_ns == 10_000_000
        assert contract.max_phase_error == 0.1
        assert contract.min_fidelity == 0.8

    def test_register_contract(self):
        """Tests contract registration"""
        manager = ContractManager()
        contract = TemporalContract.strict()

        contract_id = manager.register_contract(contract)

        assert isinstance(contract_id, str)
        assert len(contract_id) > 0

    def test_validate_execution_success(self):
        """Tests successful validation"""
        manager = ContractManager()
        contract = TemporalContract.strict()
        contract_id = manager.register_contract(contract)

        # Execution within limits
        valid = manager.validate_execution(
            contract_id,
            duration_ns=500_000,  # 0.5ms < 1ms
            phase_error=0.005,  # 0.5% < 1%
            fidelity=0.97,  # 97% > 95%
        )

        assert valid

    def test_validate_execution_failure(self):
        """Tests failed validation"""
        manager = ContractManager()
        contract = TemporalContract.strict()
        contract_id = manager.register_contract(contract)

        # Execution outside limits
        valid = manager.validate_execution(
            contract_id,
            duration_ns=5_000_000,  # 5ms > 1ms
            phase_error=0.05,  # 5% > 1%
            fidelity=0.8,  # 80% < 95%
        )

        assert not valid

    def test_statistics(self):
        """Tests statistics"""
        manager = ContractManager()
        contract = TemporalContract.strict()
        contract_id = manager.register_contract(contract)

        # Mixed validations
        manager.validate_execution(contract_id, 500_000, 0.005, 0.97)  # Valid
        manager.validate_execution(contract_id, 5_000_000, 0.05, 0.8)  # Invalid
        manager.validate_execution(contract_id, 500_000, 0.005, 0.97)  # Valid

        stats = manager.get_statistics()

        assert stats["total_validations"] == 3
        assert stats["total_violations"] == 1
        assert abs(stats["success_rate"] - 2 / 3) < 0.01


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust module required")
class TestShadowCollector:
    """Tests for ShadowCollector (Rust)"""

    def test_creation(self):
        """Tests collector creation"""
        collector = ShadowCollector(100, 0.95)

        stats = collector.get_statistics()
        assert stats["total_shadows"] == 0
        assert stats["total_snapshots"] == 0

    def test_create_shadow(self):
        """Tests shadow creation"""
        collector = ShadowCollector(100, 0.95)

        shadow_id = collector.create_shadow(num_qubits=2)

        assert isinstance(shadow_id, str)
        assert len(shadow_id) > 0

    def test_add_snapshots(self):
        """Tests snapshot addition"""
        collector = ShadowCollector(10, 0.95)
        shadow_id = collector.create_shadow(num_qubits=2)

        # Add snapshots
        for i in range(10):
            outcome = [bool(i % 2), bool((i + 1) % 2)]
            collector.add_snapshot_to_shadow(shadow_id, outcome)

        shadow = collector.get_shadow(shadow_id)

        assert shadow.collected_snapshots() == 10
        assert shadow.is_complete()

    def test_snapshot_validation(self):
        """Tests snapshot validation"""
        collector = ShadowCollector(10, 0.95)
        shadow_id = collector.create_shadow(num_qubits=2)

        # Snapshot with the wrong number of qubits should fail
        with pytest.raises(ValueError):
            collector.add_snapshot_to_shadow(shadow_id, [True, False, True])

    def test_confidence_calculation(self):
        """Tests confidence calculation"""
        collector = ShadowCollector(100, 0.95)
        shadow_id = collector.create_shadow(num_qubits=2)

        # Initial confidence
        shadow = collector.get_shadow(shadow_id)
        assert shadow.confidence() == 0.0

        # Add some snapshots
        for i in range(50):
            collector.add_snapshot_to_shadow(shadow_id, [bool(i % 2), False])

        shadow = collector.get_shadow(shadow_id)
        confidence = shadow.confidence()

        # confidence = 1 - 1/(1 + √50) ≈ 0.88
        assert 0.85 < confidence < 0.90


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust module required")
def test_kernel_info():
    """Tests kernel info retrieval"""
    info = get_kernel_info()

    assert "version" in info
    assert "rust_version" in info
    assert "build_type" in info
    assert "modules" in info

    assert info["version"] == "0.1.0"
    assert isinstance(info["modules"], list)


@pytest.mark.skipif(not RUST_AVAILABLE, reason="Rust module required")
def test_benchmark():
    """Tests benchmark function"""
    iterations = 10_000

    elapsed = benchmark_emf(iterations)

    assert elapsed > 0
    assert elapsed < 1.0  # Should be < 1s for 10k iterations
