# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# teleportation.py — Quantum Teleportation — full Python protocol implementation
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 06-07-2023
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Quantum Teleportation Protocol

Implements the quantum teleportation protocol:
1. Alice and Bob share an entangled pair (|Φ+⟩)
2. Alice wants to teleport state |ψ⟩ to Bob
3. Alice performs a Bell measurement (2 classical bits)
4. Bob applies corrections based on the bits
5. Bob obtains |ψ⟩

Initial state:
    |ψ⟩_A ⊗ |Φ+⟩_{AB}

Where |Φ+⟩ = (|00⟩ + |11⟩)/√2

After measurement and correction:
    |ψ⟩_B (original state transferred to Bob)
"""

from typing import Tuple, Optional
import numpy as np
from dataclasses import dataclass

from lightqos.core.quantum_state import QuantumState
from lightqos.core.gates import QuantumGate, CNOT, H, X, Z


@dataclass
class TeleportationResult:
    """Teleportation result"""
    measurement_bits: Tuple[int, int]  # (bit_0, bit_1)
    correction_applied: str            # "I", "X", "Z", "XZ"
    final_fidelity: float              # Final-state fidelity
    success: bool


class QuantumTeleportation:
    """
    Quantum Teleportation Protocol

    Usage:
        protocol = QuantumTeleportation()
        state_to_teleport = QuantumState.from_bloch(theta=np.pi/4, phi=0)
        result = protocol.teleport(state_to_teleport)
    """

    def __init__(self):
        self.measurements_history = []
        self.success_count = 0
        self.total_attempts = 0

    def create_bell_pair(self) -> QuantumState:
        """
        Creates Bell pair |Φ+⟩ = (|00⟩ + |11⟩)/√2

        Returns:
            QuantumState of 2 entangled qubits
        """
        # Starts in |00⟩
        state = QuantumState(num_qubits=2)

        # Applies H to qubit 0
        state = H().apply(state, [0])

        # Applies CNOT(0, 1)
        state = CNOT().apply(state, [0, 1])

        return state

    def bell_measurement(
        self,
        state: QuantumState,
        qubit_a: int,
        qubit_b: int
    ) -> Tuple[int, int]:
        """
        Performs a Bell measurement on two qubits

        Bell measurement projects onto one of the 4 Bell bases:
        - |Φ+⟩ = (|00⟩ + |11⟩)/√2  → bits (0, 0)
        - |Φ-⟩ = (|00⟩ - |11⟩)/√2  → bits (0, 1)
        - |Ψ+⟩ = (|01⟩ + |10⟩)/√2  → bits (1, 0)
        - |Ψ-⟩ = (|01⟩ - |10⟩)/√2  → bits (1, 1)

        Args:
            state: Quantum state
            qubit_a: First qubit
            qubit_b: Second qubit

        Returns:
            Tuple (bit_0, bit_1) indicating the Bell basis
        """
        # Apply CNOT(qubit_a, qubit_b)
        state = CNOT().apply(state, [qubit_a, qubit_b])

        # Apply H(qubit_a)
        state = H().apply(state, [qubit_a])

        # Measure both qubits in the Z basis
        bit_a = state.measure(qubit_a)
        bit_b = state.measure(qubit_b)

        return (bit_a, bit_b)

    def apply_correction(
        self,
        state: QuantumState,
        qubit: int,
        bits: Tuple[int, int]
    ) -> Tuple[QuantumState, str]:
        """
        Applies Pauli correction based on the measured bits

        Corrections:
        - (0, 0) → I  (identity, nothing to do)
        - (0, 1) → X  (bit flip)
        - (1, 0) → Z  (phase flip)
        - (1, 1) → XZ (bit + phase flip)

        Args:
            state: Quantum state
            qubit: Qubit to apply the correction to
            bits: Measurement bits (bit_0, bit_1)

        Returns:
            (corrected_state, correction_name)
        """
        bit_0, bit_1 = bits

        if bit_0 == 0 and bit_1 == 0:
            # No correction required
            return state, "I"

        elif bit_0 == 0 and bit_1 == 1:
            # Apply X
            state = X().apply(state, [qubit])
            return state, "X"

        elif bit_0 == 1 and bit_1 == 0:
            # Apply Z
            state = Z().apply(state, [qubit])
            return state, "Z"

        else:  # bit_0 == 1 and bit_1 == 1
            # Apply Z and X
            state = Z().apply(state, [qubit])
            state = X().apply(state, [qubit])
            return state, "XZ"

    def teleport(
        self,
        state_to_teleport: QuantumState,
        use_noise: bool = False,
        noise_level: float = 0.01
    ) -> TeleportationResult:
        """
        Executes the full teleportation protocol

        Steps:
        1. Create a Bell pair between Alice and Bob
        2. Alice has state |ψ⟩ + half of the Bell pair
        3. Alice performs a Bell measurement
        4. Bob receives 2 classical bits
        5. Bob applies correction

        Args:
            state_to_teleport: 1-qubit state to teleport
            use_noise: Simulate channel noise?
            noise_level: Noise level (0.0 to 1.0)

        Returns:
            TeleportationResult with measured bits and fidelity
        """
        self.total_attempts += 1

        # 1. Create 3-qubit system
        # Qubit 0: Alice's state (|ψ⟩)
        # Qubits 1,2: Bell pair (Alice has 1, Bob has 2)

        # Initial state: |ψ⟩ ⊗ |00⟩
        full_state = QuantumState(num_qubits=3)

        # Copy Alice's state to qubit 0
        full_state.amplitudes = np.kron(
            state_to_teleport.amplitudes,
            full_state.amplitudes[::2]  # |00⟩ of the other qubits
        )

        # 2. Create Bell pair between qubits 1 and 2
        full_state = H().apply(full_state, [1])
        full_state = CNOT().apply(full_state, [1, 2])

        # 3. Alice performs Bell measurement on qubits 0 and 1
        bits = self.bell_measurement(full_state, 0, 1)
        self.measurements_history.append(bits)

        # 4. Bob applies correction on qubit 2 based on the bits
        final_state, correction = self.apply_correction(full_state, 2, bits)

        # Simulate noise if requested
        if use_noise:
            final_state = self._apply_noise(final_state, 2, noise_level)

        # 5. Calculate fidelity
        # Extract state of qubit 2 (Bob)
        bob_state = final_state.partial_trace([0, 1])
        fidelity = self._calculate_fidelity(state_to_teleport, bob_state)

        success = fidelity > 0.95  # Success threshold
        if success:
            self.success_count += 1

        return TeleportationResult(
            measurement_bits=bits,
            correction_applied=correction,
            final_fidelity=fidelity,
            success=success
        )

    def _apply_noise(
        self,
        state: QuantumState,
        qubit: int,
        noise_level: float
    ) -> QuantumState:
        """Applies depolarizing noise"""
        # Simplification: reduce amplitude by a noise factor
        state.amplitudes *= (1.0 - noise_level)
        state.normalize()
        return state

    def _calculate_fidelity(
        self,
        state1: QuantumState,
        state2: QuantumState
    ) -> float:
        """
        Calculates fidelity between two states
        F = |⟨ψ₁|ψ₂⟩|²
        """
        overlap = np.vdot(state1.amplitudes, state2.amplitudes)
        fidelity = np.abs(overlap) ** 2
        return float(fidelity)

    def get_statistics(self) -> dict:
        """Returns protocol statistics"""
        return {
            "total_attempts": self.total_attempts,
            "successful": self.success_count,
            "success_rate": self.success_count / max(1, self.total_attempts),
            "measurements": self.measurements_history[-10:],  # Last 10
        }


# ============================================================================
# USAGE EXAMPLE
# ============================================================================

if __name__ == "__main__":
    print("=== Quantum Teleportation Protocol ===\n")

    # Create protocol
    protocol = QuantumTeleportation()

    # State to teleport: |+⟩ = (|0⟩ + |1⟩)/√2
    state = QuantumState.from_bloch(theta=np.pi/2, phi=0)

    print(f"Original state: {state}")
    print(f"Amplitudes: {state.amplitudes}\n")

    # Execute teleportation
    result = protocol.teleport(state)

    print(f"Measured bits: {result.measurement_bits}")
    print(f"Applied correction: {result.correction_applied}")
    print(f"Fidelity: {result.final_fidelity:.4f}")
    print(f"Success: {result.success}\n")

    # Test with noise
    print("=== With Noise (1%) ===\n")
    result_noisy = protocol.teleport(state, use_noise=True, noise_level=0.01)

    print(f"Fidelity with noise: {result_noisy.final_fidelity:.4f}")
    print(f"Success: {result_noisy.success}\n")

    # Statistics
    stats = protocol.get_statistics()
    print(f"=== Statistics ===")
    print(f"Attempts: {stats['total_attempts']}")
    print(f"Success rate: {stats['success_rate']:.1%}")
