#!/usr/bin/env python3
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# pyo3_bindings_demo.py — PyO3 Bindings Demo — direct Rust kernel access from Python
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 30-09-2021
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Example: PyO3 Bindings - Python ↔ Rust

Demonstrates the use of PyO3 bindings for critical-performance code.
Compairs pure Python vs Rust.
"""

import time 
import numpy as np 

# Import Rust module (bindings PyO3)
try :
    from lightqos ._rust import (
    EMFManager ,
    EntangledPair ,
    ContractManager ,
    TemporalContract ,
    HarmonicScheduler ,
    ShadowCollector ,
    QuantumShadow ,
    get_kernel_info ,
    benchmark_emf ,
    )
    RUST_AVAILABLE =True 
except ImportError :
    print ("⚠️  Rust module not available. Compile with:")
    print ("   pip install -e .")
    RUST_AVAILABLE =False 


def demo_emf_manager ():
    """Demonstrates EMFManager (Rust)"""
    print ("="*60 )
    print ("1️⃣  EMF Manager (Rust)")
    print ("="*60 )

    if not RUST_AVAILABLE :
        return 

        # Create manager
    emf =EMFManager (max_pairs =1000 ,recycling_threshold =0.5 )
    print (f"Manager criado: {emf }")

    # Generate pairs
    print ("\n📊 Generating 100 entangled pairs...")
    start =time .perf_counter ()

    pair_ids =[]
    for i in range (100 ):
        pair_id =emf .generate_pair (
        qubit_a =i ,
        qubit_b =i +100 ,
        fidelity =0.95 
        )
        pair_ids .append (pair_id )

    elapsed =time .perf_counter ()-start 
    print (f"✓ 100 pares gerados em {elapsed *1000 :.2f}ms")
    print (f"  Fidelidade média: {emf .avg_fidelity ():.3f}")

    # Envelhecer pairs
    print ("\n⏰ Aging pairs (10 timesteps)...")
    for _ in range (10 ):
        emf .age_all_pairs ()

    print (f"  Fidelidade após aging: {emf .avg_fidelity ():.3f}")

    # Reciclar
    print ("\n♻️  Recycling low-fidelity pairs...")
    recycled =emf .recycle ()
    print (f"  Pares reciclados: {recycled }")
    print (f"  Pares restantes: {emf .num_pairs ()}")

    # Statistics
    stats =emf .get_statistics ()
    print ("\n📈 Statistics:")
    for key ,value in stats .items ():
        print (f"  {key }: {value }")


def demo_contract_manager ():
    """Demonstrates ContractManager (Rust)"""
    print ("\n"+"="*60 )
    print ("2️⃣  Contract Manager (Rust)")
    print ("="*60 )

    if not RUST_AVAILABLE :
        return 

        # Create manager
    manager =ContractManager ()
    print (f"Manager criado: {manager }")

    # Create contracts
    strict =TemporalContract .strict ()
    relaxed =TemporalContract .relaxed ()

    print (f"\nContrato strict: {strict }")
    print (f"Contrato relaxed: {relaxed }")

    # Register
    strict_id =manager .register_contract (strict )
    relaxed_id =manager .register_contract (relaxed )

    # Validar execuções
    print ("\n✅ Validating executions...")

    # Good execution (strict)
    valid =manager .validate_execution (
    contract_id =strict_id ,
    duration_ns =500_000 ,# 0.5ms
    phase_error =0.005 ,# 0.5%
    fidelity =0.97 # 97%
    )
    print (f"  Execução 1 (strict): {'✓ VALID'if valid else '✗ INVALID'}")

    # Bad execution (strict)
    valid =manager .validate_execution (
    contract_id =strict_id ,
    duration_ns =5_000_000 ,# 5ms (too slow!)
    phase_error =0.05 ,# 5%
    fidelity =0.8 # 80%
    )
    print (f"  Execução 2 (strict): {'✓ VALID'if valid else '✗ INVALID'}")

    # Execution OK (relaxed)
    valid =manager .validate_execution (
    contract_id =relaxed_id ,
    duration_ns =5_000_000 ,
    phase_error =0.05 ,
    fidelity =0.85 
    )
    print (f"  Execução 3 (relaxed): {'✓ VALID'if valid else '✗ INVALID'}")

    # Statistics
    stats =manager .get_statistics ()
    print ("\n📈 Statistics:")
    for key ,value in stats .items ():
        print (f"  {key }: {value }")


def demo_shadow_collector ():
    """Demonstrates ShadowCollector (Rust)"""
    print ("\n"+"="*60 )
    print ("3️⃣  Shadow Collector (Rust)")
    print ("="*60 )

    if not RUST_AVAILABLE :
        return 

        # Create collector
    collector =ShadowCollector (
    snapshots_per_shadow =100 ,
    target_confidence =0.95 
    )
    print (f"Collector criado: {collector }")

    # Create shadow
    shadow_id =collector .create_shadow (num_qubits =4 )
    print (f"\n📸 Shadow criado: {shadow_id [:16 ]}...")

    # Coletar snapshots
    print ("\n📊 Collecting 100 snapshots...")
    start =time .perf_counter ()

    for i in range (100 ):
    # Simular measurement
        outcome =[bool (i %2 ),bool ((i +1 )%2 ),bool (i %3 ),bool (i %4 )]
        collector .add_snapshot_to_shadow (shadow_id ,outcome )

    elapsed =time .perf_counter ()-start 
    print (f"✓ 100 snapshots em {elapsed *1000 :.2f}ms")

    # Verificar shadow
    shadow =collector .get_shadow (shadow_id )
    print (f"\n📈 Shadow Info:")
    print (f"  Snapshots: {shadow .collected_snapshots ()}/{shadow .num_snapshots }")
    print (f"  Confiança: {shadow .confidence ():.4f}")
    print (f"  Completo: {shadow .is_complete ()}")

    # Estimate observable
    expectation =shadow .estimate_observable ("Z_0")
    print (f"  ⟨Z₀⟩ = {expectation :.3f}")

    # Statistics
    stats =collector .get_statistics ()
    print ("\n📈 Statistics:")
    for key ,value in stats .items ():
        print (f"  {key }: {value }")


def benchmark_python_vs_rust ():
    """Compara performance Python vs Rust"""
    print ("\n"+"="*60 )
    print ("4️⃣  Benchmark: Python vs Rust")
    print ("="*60 )

    if not RUST_AVAILABLE :
        return 

    iterations =1_000_000 

    # Pure Python
    print (f"\n🐍 Python puro ({iterations :,} iterações)...")
    start =time .perf_counter ()

    for _ in range (iterations ):
        fidelity =0.95 *0.98 # Simple calculation

    python_time =time .perf_counter ()-start 
    print (f"   Tempo: {python_time :.4f}s")

    # Rust via PyO3
    print (f"\n🦀 Rust via PyO3 ({iterations :,} iterações)...")
    rust_time =benchmark_emf (iterations )
    print (f"   Tempo: {rust_time :.4f}s")

    # Comparison
    speedup =python_time /rust_time 
    print (f"\n⚡ Speedup: {speedup :.1f}x mais rápido!")
    print (f"   Rust é {((speedup -1 )*100 ):.0f}% mais rápido que Python")


def demo_kernel_info ():
    """Shows kernel information"""
    print ("\n"+"="*60 )
    print ("5️⃣  Kernel Info")
    print ("="*60 )

    if not RUST_AVAILABLE :
        return 

    info =get_kernel_info ()

    print ("\n🔧 Kernel Rust:")
    for key ,value in info .items ():
        if isinstance (value ,list ):
            print (f"  {key }:")
            for item in value :
                print (f"    • {item }")
        else :
            print (f"  {key }: {value }")


def main ():
    """Main function"""
    print ("\n"+"🔗"*30 )
    print ("  LIGHTQOS - PyO3 BINDINGS DEMO")
    print ("🔗"*30 )

    if not RUST_AVAILABLE :
        print ("\n❌ Module Rust not available!")
        print ("\nPara compilar:")
        print ("  1. Certifique-se de ter Rust instalado")
        print ("  2. Execute: pip install -e .")
        print ("  3. Execute este script novamente")
        return 

    try :
        demo_kernel_info ()
        demo_emf_manager ()
        demo_contract_manager ()
        demo_shadow_collector ()
        benchmark_python_vs_rust ()

        print ("\n"+"="*60 )
        print ("✅ Todos os demos completados com success!")
        print ("="*60 )

    except Exception as e :
        print (f"\n❌ Erro: {e }")
        import traceback 
        traceback .print_exc ()


if __name__ =="__main__":
    main ()
