# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# predictor.py — EMF Predictor — LSTM-based entanglement demand forecasting
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 15-12-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
EMF Predictor - LSTM-Based State Prediction

Uses LSTM (Long Short-Term Memory) to predict:
- Future states of the Entangled Memory Fabric (EMF)
- Fidelity degradation over time
- Pressure gradients (TUCU)
- Recycling needs

Similar to time-series forecasting, but for quantum states!

Arquitetura:
Input: Historical sequence of EMF states
       [state_t-10, state_t-9, ..., state_t-1, state_t]

LSTM Layers (bidirecional)

Output: Future-state predictions
        [state_t+1, state_t+2, ..., state_t+n]
"""

from typing import List, Dict, Any, Optional
import numpy as np
from dataclasses import dataclass
from enum import Enum


class ThermodynamicPhase(Enum):
    """TUCU thermodynamic phases"""

    GENERATION = "4+"  # Generation (ergotropy > 0.8)
    ACTIVE = "1+"  # Active (in use, fidelity > 0.85)
    DEGRADATION = "1+"  # Degradation (0.5 < fidelity < 0.85)
    RADIATION = "1-"  # Radiation (fidelity < 0.5)
    INERTIA = "0="  # Inertia (rest)


@dataclass
class EMFState:
    """EMF state at a timestep"""

    timestamp: int
    fidelity: float  # 0.0 to 1.0
    ergotropy: float  # Useful work available
    pressure: float  # TUCU pressure (load/capacity)
    phase: ThermodynamicPhase
    num_pairs: int  # Active pairs
    avg_age: float  # Average age of pairs


class EMFPredictor:
    """
    LSTM predictor for EMF states

    Simplified implementation without PyTorch/TensorFlow.
    In production: use a real LSTM with memory cells.

    Predicts:
    - Future fidelity
    - Phase transitions
    - Recycling moments
    - Pressure overflows
    """

    def __init__(
        self,
        hidden_size: int = 64,  # Hidden-state size
        num_layers: int = 2,  # LSTM layers
        lookback: int = 20,  # Observation window
        bidirectional: bool = True,  # Bidirectional LSTM
    ):
        self.hidden_size = hidden_size
        self.num_layers = num_layers
        self.lookback = lookback
        self.bidirectional = bidirectional

        # Internal state
        self.history: List[EMFState] = []
        self.predictions_made = 0

        # "Pesos" LSTM (simplification)
        self.lstm_weights = None

        # Prediction statistics
        self.prediction_errors = []

    async def predict(self, current_state: EMFState, steps_ahead: int = 10) -> List[EMFState]:
        """
        Predicts future EMF states

        Process:
        1. Add the current state to the history
        2. Prepare the input sequence (lookback)
        3. Forward pass LSTM
        4. Generate predictions

        Args:
            current_state: Current state of the EMF
            steps_ahead: Number of steps to predict

        Returns:
            List of predicted states
        """
        # Add to history
        self.history.append(current_state)

        # Check whether there is enough history
        if len(self.history) < self.lookback:
            # Trivial prediction: keep the state
            return [current_state] * steps_ahead

        # Prepare input
        input_sequence = self._prepare_sequence()

        # LSTM forward (simplificado)
        hidden_state = self._lstm_forward(input_sequence)

        # Generate predictions
        predictions = self._generate_predictions(hidden_state, steps_ahead)

        self.predictions_made += 1

        return predictions

    def _prepare_sequence(self) -> np.ndarray:
        """
        Prepares the input sequence for the LSTM

        Converts states into numerical vectors:
        [fidelity, ergotropy, pressure, phase_encoded, num_pairs_norm, avg_age_norm]

        Returns:
            Array (lookback, feature_dim)
        """
        recent_states = self.history[-self.lookback :]

        sequence = []
        for state in recent_states:
            # Encode phase (one-hot simplificado)
            phase_code = {
                ThermodynamicPhase.GENERATION: 0.0,
                ThermodynamicPhase.ACTIVE: 0.25,
                ThermodynamicPhase.DEGRADATION: 0.5,
                ThermodynamicPhase.RADIATION: 0.75,
                ThermodynamicPhase.INERTIA: 1.0,
            }[state.phase]

            # Normalize values
            num_pairs_norm = min(state.num_pairs / 100.0, 1.0)
            age_norm = min(state.avg_age / 1000.0, 1.0)

            # Feature vector
            features = [
                state.fidelity,
                state.ergotropy,
                state.pressure,
                phase_code,
                num_pairs_norm,
                age_norm,
            ]

            sequence.append(features)

        return np.array(sequence)

    def _lstm_forward(self, input_sequence: np.ndarray) -> np.ndarray:
        """
        LSTM forward pass (simplified)

        In production: real LSTM cells with forget, input and output gates
        Here: simplification with a weighted average

        Returns:
            Hidden state final
        """
        # Simplification: weighted average with higher weight for recent timesteps
        weights = np.exp(np.linspace(-1, 0, self.lookback))
        weights /= weights.sum()

        # Weighted average
        hidden = np.average(input_sequence, axis=0, weights=weights)

        return hidden

    def _generate_predictions(self, hidden_state: np.ndarray, steps: int) -> List[EMFState]:
        """
        Generates predictions from the hidden state

        Applies temporal degradation:
        - Fidelity decreases exponentially
        - Ergotropy decreases
        - Pressure increases if pairs are not recycled
        """
        predictions = []

        # Base state (last known)
        base_state = self.history[-1]

        for step in range(1, steps + 1):
            # Temporal degradation
            decay_factor = np.exp(-0.01 * step)  # Exponential decay

            # Predict fidelity
            predicted_fidelity = base_state.fidelity * decay_factor

            # Predict ergotropy
            predicted_ergotropy = base_state.ergotropy * decay_factor

            # Predict pressure (increases over time)
            predicted_pressure = min(base_state.pressure + 0.01 * step, 1.0)

            # Determine phase
            predicted_phase = self._predict_phase(predicted_fidelity, predicted_ergotropy)

            # Number of pairs (may decrease due to consumption)
            predicted_num_pairs = max(int(base_state.num_pairs * (1.0 - 0.005 * step)), 0)

            # Average age increases
            predicted_avg_age = base_state.avg_age + step

            # Create predicted state
            predicted_state = EMFState(
                timestamp=base_state.timestamp + step,
                fidelity=predicted_fidelity,
                ergotropy=predicted_ergotropy,
                pressure=predicted_pressure,
                phase=predicted_phase,
                num_pairs=predicted_num_pairs,
                avg_age=predicted_avg_age,
            )

            predictions.append(predicted_state)

        return predictions

    def _predict_phase(self, fidelity: float, ergotropy: float) -> ThermodynamicPhase:
        """Predicts the thermodynamic phase"""
        if ergotropy > 0.8 and fidelity > 0.95:
            return ThermodynamicPhase.GENERATION
        elif fidelity > 0.85:
            return ThermodynamicPhase.ACTIVE
        elif fidelity > 0.5:
            return ThermodynamicPhase.DEGRADATION
        elif fidelity > 0.1:
            return ThermodynamicPhase.RADIATION
        else:
            return ThermodynamicPhase.INERTIA

    # ========================================================================
    # ANALYSIS AND ALERTS
    # ========================================================================

    def detect_anomalies(self, predictions: List[EMFState], threshold: float = 0.7) -> List[str]:
        """
        Detects anomalies in the predictions

        Alerts:
        - Fidelity below the threshold
        - Pressure overflow
        - Transition to radiation/inertia
        - Pair depletion
        """
        alerts = []

        for i, state in enumerate(predictions):
            # Alert: low fidelity
            if state.fidelity < threshold:
                alerts.append(
                    f"Step {i + 1}: Low fidelity ({state.fidelity:.2f}) - Recycling recommended"
                )

            # Alert: pressure overflow
            if state.pressure > 0.9:
                alerts.append(
                    f"Step {i + 1}: High pressure ({state.pressure:.2f}) - "
                    f"Capacity limit approaching"
                )

            # Alert: transition to radiation
            if state.phase == ThermodynamicPhase.RADIATION:
                alerts.append(f"Step {i + 1}: Entering RADIATION phase - Immediate action required")

            # Alert: depletion
            if state.num_pairs < 10:
                alerts.append(
                    f"Step {i + 1}: Low pair count ({state.num_pairs}) - Generation needed"
                )

        return alerts

    def estimate_recycling_time(
        self, predictions: List[EMFState], fidelity_threshold: float = 0.5
    ) -> Optional[int]:
        """
        Estimates when recycling will be needed

        Returns:
            Number of steps until recycling, or None if not required
        """
        for i, state in enumerate(predictions):
            if state.fidelity < fidelity_threshold:
                return i + 1

        return None

    def get_statistics(self) -> Dict[str, Any]:
        """Returns predictor statistics"""
        avg_error = np.mean(self.prediction_errors) if self.prediction_errors else 0.0

        return {
            "hidden_size": self.hidden_size,
            "num_layers": self.num_layers,
            "lookback_window": self.lookback,
            "bidirectional": self.bidirectional,
            "history_length": len(self.history),
            "predictions_made": self.predictions_made,
            "avg_prediction_error": avg_error,
        }


# ============================================================================
# USAGE EXAMPLE
# ============================================================================

if __name__ == "__main__":
    import asyncio

    async def main():
        print("=== EMF Predictor (LSTM) ===\n")

        # Create predictor
        predictor = EMFPredictor(lookback=10)

        # Simulate state history
        print("Creating state history...")
        for t in range(20):
            state = EMFState(
                timestamp=t,
                fidelity=0.95 - 0.01 * t,  # Degradation gradual
                ergotropy=0.9 - 0.02 * t,
                pressure=0.3 + 0.02 * t,
                phase=ThermodynamicPhase.ACTIVE,
                num_pairs=100 - t,
                avg_age=t * 10,
            )
            predictor.history.append(state)

        print(f"History: {len(predictor.history)} states\n")

        # Current state
        current = EMFState(
            timestamp=20,
            fidelity=0.75,
            ergotropy=0.5,
            pressure=0.7,
            phase=ThermodynamicPhase.DEGRADATION,
            num_pairs=80,
            avg_age=200,
        )

        print("Current state (t=20):")
        print(f"  Fidelity: {current.fidelity:.2f}")
        print(f"  Pressure: {current.pressure:.2f}")
        print(f"  Phase: {current.phase.name}\n")

        # Prediction
        print("Predicting the next 10 steps...")
        predictions = await predictor.predict(current, steps_ahead=10)

        print("\nPredictions:")
        for i, pred in enumerate(predictions):
            print(
                f"  t+{i + 1}: F={pred.fidelity:.3f}, "
                f"P={pred.pressure:.3f}, "
                f"phase={pred.phase.name}"
            )

        # Detectar anomalys
        print("\n=== Alerts ===")
        alerts = predictor.detect_anomalies(predictions)
        if alerts:
            for alert in alerts:
                print(f"⚠️  {alert}")
        else:
            print("✓ No anomaly detected")

        # Estimar recycling
        print("\n=== Recycling ===")
        recycle_time = predictor.estimate_recycling_time(predictions)
        if recycle_time:
            print(f"Recycling recommended in: {recycle_time} steps")
        else:
            print("Recycling not required within the predicted horizon")

        # Statistics
        print("\n=== Statistics ===")
        stats = predictor.get_statistics()
        for key, value in stats.items():
            print(f"{key}: {value}")

    asyncio.run(main())
