# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# qkd.py — QKD Protocol — Quantum Key Distribution (BB84) Python implementation
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 06-08-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Quantum Key Distribution (QKD) - BB84 Protocol

Implements the BB84 protocol (Bennett & Brassard, 1984):
1. Alice generates random bits and chooses random bases
2. Alice sends qubits to Bob
3. Bob measures in random bases
4. Alice and Bob compare bases (classical channel)
5. They discard measurements with incompatible bases
6. Eavesdropping verification (QBER)
7. Error correction and privacy amplification

Security:
- Any Eve measurement disturbs the qubits
- QBER (Quantum Bit Error Rate) detects eavesdropping
- Theoretically secure information against quantum attacks
"""

from typing import List, Tuple, Optional
import numpy as np
from dataclasses import dataclass
from enum import Enum


class Basis(Enum):
    """Measurement bases"""
    RECTILINEAR = "Z"  # Computational basis (Z): |0⟩, |1⟩
    DIAGONAL = "X"     # Diagonal basis (X): |+⟩, |-⟩


@dataclass
class QKDResult:
    """Key distribution result"""
    raw_key_length: int           # Raw key size
    sifted_key_length: int        # After sifting (basis comparison)
    final_key_length: int         # After error correction
    qber: float                   # Quantum Bit Error Rate
    secure: bool                  # Is the key secure?
    eavesdropping_detected: bool  # Was Eve detected?
    key: Optional[List[int]]      # Final key (if secure)


class BB84Protocol:
    """
    BB84 Quantum Key Distribution Protocol

    Usage:
        protocol = BB84Protocol(key_length=256)
        result = protocol.distribute_key()

        if result.secure:
            key = result.key
            # Use key for secure communication
    """

    def __init__(
        self,
        key_length: int = 256,
        qber_threshold: float = 0.11,  # Security threshold
        sample_size: int = None
    ):
        """
        Args:
            key_length: Desired final key size
            qber_threshold: Maximum acceptable QBER (default: 11%)
            sample_size: Sample size for verification (None = automatic)
        """
        self.key_length = key_length
        self.qber_threshold = qber_threshold
        self.sample_size = sample_size or max(100, key_length // 4)

        # History
        self.distributions = []

    def distribute_key(
        self,
        channel_error_rate: float = 0.0,
        eavesdropper_present: bool = False,
        eve_strategy: str = "intercept_resend"
    ) -> QKDResult:
        """
        Executes the full distribution protocol

        Args:
            channel_error_rate: Channel error rate (0.0 to 1.0)
            eavesdropper_present: Simulate Eve's presence?
            eve_strategy: Eve strategy ("intercept_resend", "measure_resend")

        Returns:
            QKDResult with key and statistics
        """
        # 1. PREPARATION (Alice)
        alice_bits, alice_bases, qubits = self._alice_prepare()

        # 2. TRANSMISSION (Quantum channel)
        if eavesdropper_present:
            qubits = self._eve_intercept(qubits, eve_strategy)

        # Simulate channel error
        if channel_error_rate > 0:
            qubits = self._apply_channel_noise(qubits, channel_error_rate)

        # 3. MEASUREMENT (Bob)
        bob_bits, bob_bases = self._bob_measure(qubits)

        # 4. SIFTING (Basis comparison through classical channel)
        sifted_alice_key, sifted_bob_key = self._sifting(
            alice_bits, alice_bases,
            bob_bits, bob_bases
        )

        # 5. EAVESDROPPING VERIFICATION
        qber, eve_detected = self._check_eavesdropping(
            sifted_alice_key,
            sifted_bob_key
        )

        # 6. ERROR CORRECTION AND PRIVACY AMPLIFICATION
        if not eve_detected and qber < self.qber_threshold:
            final_key = self._error_correction_and_privacy(
                sifted_alice_key,
                sifted_bob_key
            )
            secure = True
        else:
            final_key = None
            secure = False

        result = QKDResult(
            raw_key_length=len(alice_bits),
            sifted_key_length=len(sifted_alice_key),
            final_key_length=len(final_key) if final_key else 0,
            qber=qber,
            secure=secure,
            eavesdropping_detected=eve_detected,
            key=final_key
        )

        self.distributions.append(result)
        return result

    def _alice_prepare(self) -> Tuple[List[int], List[Basis], List[np.ndarray]]:
        """
        Alice prepares random qubits

        Returns:
            (bits, bases, qubits)
        """
        # Generate more bits than needed (overhead for sifting)
        n = self.key_length * 4

        # Random bits
        bits = [np.random.randint(0, 2) for _ in range(n)]

        # Random bases
        bases = [
            Basis.RECTILINEAR if np.random.rand() < 0.5 else Basis.DIAGONAL
            for _ in range(n)
        ]

        # Prepare qubits
        qubits = []
        for bit, basis in zip(bits, bases):
            qubit = self._prepare_qubit(bit, basis)
            qubits.append(qubit)

        return bits, bases, qubits

    def _prepare_qubit(self, bit: int, basis: Basis) -> np.ndarray:
        """
        Prepares the qubit in the appropriate state

        Basis Z (Rectilinear):
            bit=0 → |0⟩ = [1, 0]
            bit=1 → |1⟩ = [0, 1]

        Basis X (Diagonal):
            bit=0 → |+⟩ = (|0⟩ + |1⟩)/√2 = [1/√2, 1/√2]
            bit=1 → |-⟩ = (|0⟩ - |1⟩)/√2 = [1/√2, -1/√2]
        """
        if basis == Basis.RECTILINEAR:
            # Z basis
            if bit == 0:
                return np.array([1.0, 0.0], dtype=complex)
            else:
                return np.array([0.0, 1.0], dtype=complex)
        else:
            # X basis
            if bit == 0:
                return np.array([1.0, 1.0], dtype=complex) / np.sqrt(2)
            else:
                return np.array([1.0, -1.0], dtype=complex) / np.sqrt(2)

    def _bob_measure(self, qubits: List[np.ndarray]) -> Tuple[List[int], List[Basis]]:
        """
        Bob measures qubits in random bases

        Returns:
            (measured bits, bases used)
        """
        bits = []
        bases = []

        for qubit in qubits:
            # Choose random basis
            basis = Basis.RECTILINEAR if np.random.rand() < 0.5 else Basis.DIAGONAL
            bases.append(basis)

            # Measure
            bit = self._measure_qubit(qubit, basis)
            bits.append(bit)

        return bits, bases

    def _measure_qubit(self, qubit: np.ndarray, basis: Basis) -> int:
        """Measures a qubit in the specified basis"""
        if basis == Basis.RECTILINEAR:
            # Measure in Z basis
            prob_0 = np.abs(qubit[0]) ** 2
            return 0 if np.random.rand() < prob_0 else 1
        else:
            # Measure in X basis
            # Transform to X basis
            plus = np.array([1.0, 1.0], dtype=complex) / np.sqrt(2)
            minus = np.array([1.0, -1.0], dtype=complex) / np.sqrt(2)

            prob_plus = np.abs(np.vdot(plus, qubit)) ** 2
            return 0 if np.random.rand() < prob_plus else 1

    def _sifting(
        self,
        alice_bits: List[int],
        alice_bases: List[Basis],
        bob_bits: List[int],
        bob_bases: List[Basis]
    ) -> Tuple[List[int], List[int]]:
        """
        Sifting: Alice and Bob compare bases and keep only matches

        Returns:
            (alice_key, bob_key)
        """
        sifted_alice = []
        sifted_bob = []

        for a_bit, a_basis, b_bit, b_basis in zip(
            alice_bits, alice_bases, bob_bits, bob_bases
        ):
            if a_basis == b_basis:
                # Same bases: keep
                sifted_alice.append(a_bit)
                sifted_bob.append(b_bit)

        return sifted_alice, sifted_bob

    def _check_eavesdropping(
        self,
        alice_key: List[int],
        bob_key: List[int]
    ) -> Tuple[float, bool]:
        """
        Checks for eavesdropping by comparing samples

        Returns:
            (QBER, eavesdropping_detected)
        """
        # Select random sample
        sample_indices = np.random.choice(
            len(alice_key),
            size=min(self.sample_size, len(alice_key)),
            replace=False
        )

        # Compare sample bits
        errors = 0
        for i in sample_indices:
            if alice_key[i] != bob_key[i]:
                errors += 1

        qber = errors / len(sample_indices) if sample_indices.size > 0 else 0.0
        eve_detected = qber > self.qber_threshold

        return qber, eve_detected

    def _error_correction_and_privacy(
        self,
        alice_key: List[int],
        bob_key: List[int]
    ) -> List[int]:
        """
        Error correction and privacy amplification

        Simplification: remove bits from the verification sample
        In production: use CASCADE, LDPC, etc.
        """
        # Remove bits used in verification
        # Simplification: truncate
        final_length = min(len(alice_key) - self.sample_size, self.key_length)

        # In production: apply universal hashing for privacy amplification
        final_key = alice_key[:final_length]

        return final_key

    def _eve_intercept(
        self,
        qubits: List[np.ndarray],
        strategy: str
    ) -> List[np.ndarray]:
        """
        Simulates Eve interception

        Strategies:
        - intercept_resend: Measures and resends (introduces ~25% error)
        - measure_resend: Measures in a random basis and resends
        """
        intercepted = []

        for qubit in qubits:
            # Eve measures in a random basis
            eve_basis = Basis.RECTILINEAR if np.random.rand() < 0.5 else Basis.DIAGONAL
            eve_bit = self._measure_qubit(qubit, eve_basis)

            # Eve resends (also in a random basis)
            resend_basis = Basis.RECTILINEAR if np.random.rand() < 0.5 else Basis.DIAGONAL
            new_qubit = self._prepare_qubit(eve_bit, resend_basis)

            intercepted.append(new_qubit)

        return intercepted

    def _apply_channel_noise(
        self,
        qubits: List[np.ndarray],
        error_rate: float
    ) -> List[np.ndarray]:
        """Applies channel noise"""
        noisy = []

        for qubit in qubits:
            if np.random.rand() < error_rate:
                # Qubit flip
                noisy_qubit = np.array([qubit[1], qubit[0]], dtype=complex)
            else:
                noisy_qubit = qubit

            noisy.append(noisy_qubit)

        return noisy

    def get_statistics(self) -> dict:
        """Returns distribution statistics"""
        if not self.distributions:
            return {}

        secure_count = sum(1 for d in self.distributions if d.secure)
        avg_qber = np.mean([d.qber for d in self.distributions])

        return {
            "total_distributions": len(self.distributions),
            "secure_distributions": secure_count,
            "success_rate": secure_count / len(self.distributions),
            "avg_qber": avg_qber,
            "avg_efficiency": np.mean([
                d.final_key_length / d.raw_key_length
                for d in self.distributions
                if d.final_key_length > 0
            ]) if any(d.final_key_length > 0 for d in self.distributions) else 0.0
        }


# ============================================================================
# USAGE EXAMPLE
# ============================================================================

if __name__ == "__main__":
    print("=== BB84 Quantum Key Distribution ===\n")

    # 1. Ideal scenario (without Eve, without noise)
    print("1. Ideal Scenario:")
    protocol = BB84Protocol(key_length=128)
    result = protocol.distribute_key()

    print(f"   Raw key: {result.raw_key_length} bits")
    print(f"   After sifting: {result.sifted_key_length} bits")
    print(f"   Final key: {result.final_key_length} bits")
    print(f"   QBER: {result.qber:.2%}")
    print(f"   Secure: {result.secure}")
    print(f"   Key: {result.key[:20]}...\n")

    # 2. With eavesdropping
    print("2. With Eve (intercept-resend):")
    result_eve = protocol.distribute_key(eavesdropper_present=True)

    print(f"   QBER: {result_eve.qber:.2%}")
    print(f"   Eve detected: {result_eve.eavesdropping_detected}")
    print(f"   Secure: {result_eve.secure}\n")

    # 3. With channel noise
    print("3. With Channel Noise (5%):")
    result_noise = protocol.distribute_key(channel_error_rate=0.05)

    print(f"   QBER: {result_noise.qber:.2%}")
    print(f"   Secure: {result_noise.secure}\n")

    # Statistics
    stats = protocol.get_statistics()
    print("=== Statistics ===")
    print(f"Distributions: {stats['total_distributions']}")
    print(f"Success rate: {stats['success_rate']:.1%}")
    print(f"Average QBER: {stats['avg_qber']:.2%}")
    print(f"Efficiency: {stats['avg_efficiency']:.1%}")
