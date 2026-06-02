# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# test_protocols.py — Protocol Tests — quantum teleportation and BB84 QKD
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 18-12-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Unit Tests - Quantum Protocols

Tests: Teleportation, QKD
"""

import pytest
import numpy as np
from lightqos.protocols import QuantumTeleportation, BB84Protocol
from lightqos.core import QuantumState


class TestQuantumTeleportation:
    """Tests for quantum teleportation"""
    
    def setup_method(self):
        """Setup before each test"""
        self.protocol = QuantumTeleportation()
    
    def test_initialization(self):
        """Tests protocol initialization"""
        assert self.protocol.success_count == 0
        assert self.protocol.total_attempts == 0
        assert len(self.protocol.measurements_history) == 0
    
    def test_create_bell_pair(self):
        """Tests Bell pair creation"""
        pair = self.protocol.create_bell_pair()
        
        assert pair.num_qubits == 2
        assert pair.is_normalized()
        
        # Verify superposition
        probs = pair.probabilities()
        assert abs(probs.get('00', 0) - 0.5) < 0.01
        assert abs(probs.get('11', 0) - 0.5) < 0.01
    
    def test_teleport_simple_state(self):
        """Tests teleportation of a simple state"""
        # State |0⟩
        state = QuantumState.from_basis(0, num_qubits=1)
        
        result = self.protocol.teleport(state)
        
        assert result.success
        assert result.final_fidelity > 0.99
        assert result.measurement_bits in [(0, 0), (0, 1), (1, 0), (1, 1)]
        assert result.correction_applied in ["I", "X", "Z", "XZ"]
    
    def test_teleport_superposition(self):
        """Tests teleportation of a superposition"""
        # State |+⟩ = (|0⟩ + |1⟩)/√2
        state = QuantumState.from_bloch(theta=np.pi/2, phi=0)
        
        result = self.protocol.teleport(state)
        
        assert result.success
        assert result.final_fidelity > 0.95
    
    def test_teleport_with_noise(self):
        """Tests teleportation with noise"""
        state = QuantumState.from_bloch(theta=np.pi/4, phi=0)
        
        result = self.protocol.teleport(state, use_noise=True, noise_level=0.01)
        
        # With noise, fidelity is lower
        assert result.final_fidelity > 0.90
        assert result.final_fidelity < 0.99
    
    def test_statistics(self):
        """Tests statistics collection"""
        state = QuantumState.from_basis(0, num_qubits=1)
        
        # Run multiple teleportations
        for _ in range(10):
            self.protocol.teleport(state)
        
        stats = self.protocol.get_statistics()
        
        assert stats['total_attempts'] == 10
        assert stats['successful'] <= 10
        assert 0.0 <= stats['success_rate'] <= 1.0
        assert len(stats['measurements']) <= 10


class TestBB84Protocol:
    """Tests for BB84 QKD"""
    
    def setup_method(self):
        """Setup before each test"""
        self.protocol = BB84Protocol(key_length=128)
    
    def test_initialization(self):
        """Tests initialization"""
        assert self.protocol.key_length == 128
        assert self.protocol.qber_threshold == 0.11
        assert len(self.protocol.distributions) == 0
    
    def test_distribute_key_ideal(self):
        """Tests distribution in an ideal scenario"""
        result = self.protocol.distribute_key()
        
        assert result.secure
        assert result.final_key_length > 0
        assert result.qber < 0.05  # Very low without Eve
        assert not result.eavesdropping_detected
        assert result.key is not None
    
    def test_distribute_key_with_eve(self):
        """Tests distribution with an eavesdropper"""
        result = self.protocol.distribute_key(eavesdropper_present=True)
        
        # Eve should be detected
        assert result.eavesdropping_detected or result.qber > 0.15
        
        # Key may not be secure
        if not result.secure:
            assert result.key is None
    
    def test_distribute_key_with_noise(self):
        """Tests distribution with channel noise"""
        result = self.protocol.distribute_key(channel_error_rate=0.05)
        
        # QBER should reflect noise
        assert 0.03 < result.qber < 0.10
        
        # Should still be secure with moderate noise
        assert result.secure or result.qber > self.protocol.qber_threshold
    
    def test_sifting_reduces_key_size(self):
        """Tests that sifting reduces key size"""
        result = self.protocol.distribute_key()
        
        assert result.sifted_key_length < result.raw_key_length
        assert result.final_key_length <= result.sifted_key_length
    
    def test_statistics(self):
        """Tests statistics"""
        # Run multiple distributions
        for _ in range(5):
            self.protocol.distribute_key()
        
        stats = self.protocol.get_statistics()
        
        assert stats['total_distributions'] == 5
        assert 0 <= stats['success_rate'] <= 1.0
        assert stats['avg_qber'] >= 0.0


# ============================================================================
# FIXTURES
# ============================================================================

@pytest.fixture
def sample_quantum_state():
    """Sample quantum state"""
    return QuantumState.from_bloch(theta=np.pi/3, phi=np.pi/4)


@pytest.fixture
def teleportation_protocol():
    """Teleportation protocol"""
    return QuantumTeleportation()


# ============================================================================
# PARAMETRIZED TESTS
# ============================================================================

@pytest.mark.parametrize("theta,phi", [
    (0, 0),                    # |0⟩
    (np.pi, 0),               # |1⟩
    (np.pi/2, 0),             # |+⟩
    (np.pi/2, np.pi),         # |-⟩
    (np.pi/2, np.pi/2),       # |i+⟩
])
def test_teleport_various_states(theta, phi):
    """Tests teleportation of several states"""
    protocol = QuantumTeleportation()
    state = QuantumState.from_bloch(theta, phi)
    
    result = protocol.teleport(state)
    
    assert result.success
    assert result.final_fidelity > 0.95


@pytest.mark.parametrize("key_length", [64, 128, 256, 512])
def test_qkd_various_key_lengths(key_length):
    """Tests QKD with different key lengths"""
    protocol = BB84Protocol(key_length=key_length)
    result = protocol.distribute_key()
    
    if result.secure:
        assert len(result.key) >= key_length * 0.8  # Some loss


# ============================================================================
# MARKERS
# ============================================================================

@pytest.mark.slow
def test_teleport_many_times():
    """Slow test: multiple teleportations"""
    protocol = QuantumTeleportation()
    state = QuantumState.from_basis(0, num_qubits=1)
    
    results = [protocol.teleport(state) for _ in range(100)]
    
    success_rate = sum(r.success for r in results) / len(results)
    avg_fidelity = np.mean([r.final_fidelity for r in results])
    
    assert success_rate > 0.95
    assert avg_fidelity > 0.95


@pytest.mark.slow
def test_qkd_multiple_distributions():
    """Slow test: multiple QKD distributions"""
    protocol = BB84Protocol(key_length=128)
    
    results = [protocol.distribute_key() for _ in range(20)]
    
    secure_count = sum(r.secure for r in results)
    assert secure_count > 15  # Most should be secure
