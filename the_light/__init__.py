# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# __init__.py — The Light AI — artificial intelligence engine entry point
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 23-03-2026
# All rights reserved.
# -----------------------------------------------------------------------------

"""
THE LIGHT — LightQOS Artificial Intelligence Engine
====================================================

Integrated AI engine for quantum computing optimization.

Components:
    TranspilerOptimizer  — ML-based circuit optimization (Transformer)
    EMFPredictor         — Entanglement demand forecasting (LSTM)
    ConsciousnessMath    — 18D information integration mathematics (TUCU/IIT)
    AdaptiveCalibration  — Adaptive gate-parameter calibration

Quick use:
    >>> from the_light import TranspilerOptimizer
    >>> optimizer = TranspilerOptimizer()
    >>> optimized = optimizer.optimize(circuit, target_backend="ibm_heron")
"""

__version__ = "0.2.0"
__author__  = "Márcio Coutinho"

from .optimizer   import TranspilerOptimizer
from .predictor   import EMFPredictor
from .consciousness_math import ConsciousnessMath
from .core        import TheLight
from .transformer import CircuitTransformer

__all__ = [
    "TranspilerOptimizer",
    "EMFPredictor",
    "ConsciousnessMath",
    "TheLight",
    "CircuitTransformer",
]
