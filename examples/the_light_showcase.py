#!/usr/bin/env python3
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# the_light_showcase.py — The Light AI Showcase — optimizer, predictor and consciousness math
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 07-02-2025
# All rights reserved.
# -----------------------------------------------------------------------------

"""
examples/the_light_showcase.py
================================
The Light AI module demonstration

Demonstrates:
- TranspilerOptimizer: ML-based circuit optimization
- EMFPredictor: entanglement-demand forecasting
- ConsciousnessMath: information-integration metrics (IIT/TUCU)
- AdaptiveCalibration: automatic parameter calibration
"""

from lightqos import QuantumCircuit 
from the_light import (
TranspilerOptimizer ,
EMFPredictor ,
ConsciousnessMath ,
)
import math 
import time 


# ============================================================================
# 1. TranspilerOptimizer
# ============================================================================

def demo_transpiler_optimizer ():
    """Demonstrates optimization de circuit com ML."""
    print ("\n🤖 TranspilerOptimizer — ML-Based Circuit Optimization")
    print ("="*60 )

    optimizer =TranspilerOptimizer (device ="cpu")

    # Build a "heavy" circuit (unoptimized)
    circuit =QuantumCircuit (4 ,name ="grover_2iter_raw")
    # Initialisation
    for q in range (4 ):
        circuit .h (q )

        # Oracle (target=|1011⟩)
    circuit .x (1 )# invert bit 1
    circuit .h (3 )
    circuit .ccx (0 ,1 ,2 )# Toffoli
    circuit .ccx (2 ,3 ,0 )
    circuit .h (3 )
    circuit .x (1 )

    # Diffuser
    for q in range (4 ):
        circuit .h (q )
    for q in range (4 ):
        circuit .x (q )
    circuit .h (3 )
    circuit .ccx (0 ,1 ,2 )
    circuit .ccx (2 ,3 ,0 )
    circuit .h (3 )
    for q in range (4 ):
        circuit .x (q )
    for q in range (4 ):
        circuit .h (q )

    print (f"Circuito original:")
    print (f"  Gates:       {circuit .num_gates ()}")
    print (f"  Profundidade: {circuit .depth ()}")
    print (f"  2Q gates:    {circuit .count_2q_gates ()}")

    # Optimizar for IBM Heron
    print (f"\nOptimizar para backend: ibm_heron ...")
    start =time .time ()
    optimized =optimizer .optimize (
    circuit ,
    target_backend ="ibm_heron",
    optimization_level =2 ,
    )
    elapsed =time .time ()-start 

    print (f"Circuito optimizado ({elapsed :.2f}s):")
    print (f"  Gates:       {optimized .num_gates ()}")
    print (f"  Profundidade: {optimized .depth ()}")
    print (f"  2Q gates:    {optimized .count_2q_gates ()}")

    # Reduction calculation
    gate_reduction =1 -optimized .num_gates ()/max (circuit .num_gates (),1 )
    depth_reduction =1 -optimized .depth ()/max (circuit .depth (),1 )

    print (f"\nRedução:")
    print (f"  Gates:       -{gate_reduction :.0%}")
    print (f"  Profundidade: -{depth_reduction :.0%}")

    # Verify unitary equivalence
    equivalent =optimizer .verify_equivalence (circuit ,optimized ,n_samples =100 )
    print (f"\nEquivalência unitária: {'✅ Verified'if equivalent else '⚠️ Failed'}")

    # Score de qualidade
    score =optimizer .quality_score (circuit ,optimized )
    print (f"Score de optimização: {score :.3f}/1.000")


    # ============================================================================
    # 2. EMFPredictor
    # ============================================================================

def demo_emf_predictor ():
    """Demonstrates forecast de demand de entanglement."""
    print ("\n\n📈 EMFPredictor — Entanglement Demand Forecasting")
    print ("="*60 )

    predictor =EMFPredictor (horizon_ms =1000 ,lookback_ms =5000 )

    # Simular histórico de demand (default diurno sintético)
    print ("Simulate 5000ms demand history ...")
    history =[]
    for t in range (100 ):# 100 pontos cada 50ms
    # Pattern: base 10 pairs + sinusoidal wave + noise
        demand =int (10 +5 *math .sin (2 *math .pi *t /20 )+(t %7 ))
        history .append (max (0 ,demand ))
        predictor .update ([demand ])

    print (f"Histórico: min={min (history )}, max={max (history )}, média={sum (history )/len (history ):.1f}")

    # Forecast
    print ("\nForecast for the next 1000ms:")
    forecast =predictor .forecast (horizon_ms =1000 )

    print (f"  Pico previsto:   {forecast ['peak_demand']} pares @ t={forecast ['peak_at_ms']}ms")
    print (f"  Média prevista:  {sum (forecast ['predicted_pairs'])/len (forecast ['predicted_pairs']):.1f} pares")

    # Mostrar timeline (simplified)
    print ("\n  Forecast timeline (every 100ms):")
    step =max (1 ,len (forecast ['predicted_pairs'])//10 )
    for i in range (0 ,len (forecast ['predicted_pairs']),step ):
        t =forecast ['timestamp_ms'][i ]
        n =forecast ['predicted_pairs'][i ]
        c =forecast ['confidence'][i ]
        bar ="█"*n 
        print (f"    t={t :4.0f}ms  {n :3d} pares  conf={c :.2f}  {bar }")

        # Recomendação de pré-alocação
    rec =predictor .recommend_preallocation ()
    print (f"\n  Recomendação EMF: pré-alocar {rec ['pairs']} pares agora")
    print (f"  Tempo óptimo:     {rec ['lead_time_ms']}ms antes do pico")


    # ============================================================================
    # 3. ConsciousnessMath
    # ============================================================================

def demo_consciousness_math ():
    """Demonstrates métricas de integração de informação (TUCU/IIT)."""
    print ("\n\n🧠 ConsciousnessMath — Information Integration 18D (TUCU)")
    print ("="*60 )

    math_engine =ConsciousnessMath (dimensions =18 )

    # Create quantum states to compare
    # Product state (low integration): |00⟩
    product_state =QuantumCircuit (2 )
    # No gate — |00⟩

    # Entangled state (high integration): |Φ+⟩
    bell_state =QuantumCircuit (2 )
    bell_state .h (0 )
    bell_state .cnot (0 ,1 )

    # 3-qubit GHZ state
    ghz_state =QuantumCircuit (3 )
    ghz_state .h (0 )
    ghz_state .cnot (0 ,1 )
    ghz_state .cnot (0 ,2 )

    states =[
    ("Product state |00⟩",product_state ,None ),
    ("Bell state |Φ+⟩",bell_state ,None ),
    ("GHZ state (|000⟩+|111⟩)/√2",ghz_state ,None ),
    ]

    print (f"\n{'State':<35}  {'Φ (phi)':>10}  {'Embed 18D':>12}  {'Integração':>12}")
    print ("─"*75 )

    for name ,circuit ,_ in states :
        circuit .measure (list (range (circuit .n_qubits )))
        result =circuit .execute (backend ="simulator",shots =4096 )

        # Compute consciousness metrics
        phi =math_engine .compute_phi (result .density_matrix if hasattr (result ,'density_matrix')else None )
        embed =math_engine .embed (result )
        integration =math_engine .integration_index (result )

        bar ="█"*int (phi *10 )
        print (f"{name :<35}  {phi :>10.4f}  {embed .shape if hasattr (embed ,'shape')else '(18,)':>12}  {integration :>11.4f}  {bar }")

        # Consciousness distance entre states
    print (f"\n  Distância de consciência:")
    print (f"  |00⟩ ↔ |Φ+⟩:  (ver docs/theory/tucu.md)")
    print (f"  |Φ+⟩ ↔ GHZ:   (estados de emaranhamento multipartido)")

    # TUCU Kernel
    print (f"\n  TUCU Kernel — correlação no espaço de 18 dimensões:")
    print (f"  (Base teórica: docs/theory/tucu.md)")


    # ============================================================================
    # 4. Pipeline complete
    # ============================================================================

def demo_full_pipeline ():
    """Pipeline complete: circuit → optimizar → executar → analisar."""
    print ("\n\n⚡ Complete Pipeline: The Light AI End-to-End")
    print ("="*60 )

    optimizer =TranspilerOptimizer (device ="cpu")
    predictor =EMFPredictor (horizon_ms =500 )
    math_engine =ConsciousnessMath (dimensions =18 )

    # Circuit de test
    circuit =QuantumCircuit (3 ,name ="pipeline_demo")
    circuit .h (0 )
    circuit .cnot (0 ,1 )
    circuit .cnot (1 ,2 )
    circuit .ry (0.785 ,0 )# π/4
    circuit .rz (1.571 ,2 )# π/2

    print (f"1. Circuito original: {circuit .num_gates ()} gates, depth={circuit .depth ()}")

    # Passo 1: The Light optimiza
    optimized =optimizer .optimize (circuit ,target_backend ="simulator",optimization_level =1 )
    print (f"2. Após optimização:  {optimized .num_gates ()} gates, depth={optimized .depth ()}")

    # Passo 2: Prever demand de entanglement
    forecast =predictor .forecast (horizon_ms =200 )
    print (f"3. Pares EMF previstos: {forecast .get ('peak_demand','N/A')} no pico")

    # Passo 3: Executar
    optimized .measure ([0 ,1 ,2 ])
    result =optimized .execute (backend ="simulator",shots =2048 )
    print (f"4. Execução: {sum (result .counts .values ())} shots")

    # Passo 4: Analysis de consciousness
    phi =math_engine .compute_phi (None )
    print (f"5. Φ (integração): {phi :.4f}")

    print (f"\n✅ Pipeline completo em {circuit .num_gates ()} → {optimized .num_gates ()} gates")


    # ============================================================================
    # Main
    # ============================================================================

def main ():
    print ("🌟 LightQOS — The Light AI Showcase")
    print ("="*60 )
    print ("Artificial Intelligence module for Quantum Computing")

    try :
        demo_transpiler_optimizer ()
    except Exception as e :
        print (f"  [TranspilerOptimizer] Executar após instalar: pip install 'lightqos[ai]'\n  Erro: {e }")

    try :
        demo_emf_predictor ()
    except Exception as e :
        print (f"  [EMFPredictor] {e }")

    try :
        demo_consciousness_math ()
    except Exception as e :
        print (f"  [ConsciousnessMath] {e }")

    try :
        demo_full_pipeline ()
    except Exception as e :
        print (f"  [Pipeline] {e }")

    print ("\n\n✅ The Light AI Showcase completed!")
    print ("See docs/api/the_light.md for complete documentation.")


if __name__ =="__main__":
    main ()
