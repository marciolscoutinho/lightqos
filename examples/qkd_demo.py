#!/usr/bin/env python3
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# qkd_demo.py — QKD Demo — BB84 Quantum Key Distribution full simulation
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 25-05-2025
# All rights reserved.
# -----------------------------------------------------------------------------

"""
examples/qkd_demo.py
====================
QKD (Quantum Key Distribution) demonstration — BB84 Protocol

Demonstrates:
- Complete BB84 protocol (Alice, Bob, Eve)
- Quantum Channel with noise
- QBER (Quantum Bit Error Rate) estimation
- Eavesdropping detection (Eve)
- Integration with the LightQOS protocols module
"""

import random 
import math 
from dataclasses import dataclass ,field 
from typing import Optional 
from lightqos import QuantumCircuit ,TemporalContract 


# ============================================================================
# Constantes
# ============================================================================

BASES =["Z","X"]# Z = {|0⟩,|1⟩}, X = {|+⟩,|-⟩}
QBER_THRESHOLD =0.11 # QBER maximum tolerável sin espionagin (~11%)


# ============================================================================
# Estruturas de data
# ============================================================================

@dataclass 
class QKDResult :
    """Result of a QKD session."""
    raw_key_alice :list [int ]
    raw_key_bob :list [int ]
    sifted_key :list [int ]
    qber :float 
    eve_detected :bool 
    key_length :int 
    efficiency :float # key bits per sent qubit


    # ============================================================================
    # Pairticipantes
    # ============================================================================

class Alice :
    """Alice — qubit sender."""

    def __init__ (self ,n_bits :int ,seed :int =42 ):
        random .seed (seed )
        self .n_bits =n_bits 
        self .bits =[random .randint (0 ,1 )for _ in range (n_bits )]
        self .bases =[random .choice (BASES )for _ in range (n_bits )]
        print (f"Alice: preparar {n_bits } qubits")

    def prepare_qubit (self ,i :int )->QuantumCircuit :
        """Prepairs qubit i in the chosen basis."""
        circuit =QuantumCircuit (1 ,name =f"alice_q{i }")
        bit =self .bits [i ]
        base =self .bases [i ]

        if bit ==1 :
            circuit .x (0 )# |1⟩ se bit=1

        if base =="X":
            circuit .h (0 )# Hadamard para base X

        return circuit 

    def prepare_all (self )->list [QuantumCircuit ]:
        return [self .prepare_qubit (i )for i in range (self .n_bits )]


class Bob :
    """Bob — qubit receiver."""

    def __init__ (self ,n_bits :int ,seed :int =123 ):
        random .seed (seed )
        self .n_bits =n_bits 
        self .bases =[random .choice (BASES )for _ in range (n_bits )]
        self .results =[]
        print (f"Bob: aguardar {n_bits } qubits")

    def measure_qubit (self ,circuit :QuantumCircuit ,i :int )->int :
        """Mede qubit in the base escolhida por Bob."""
        meas_circuit =circuit .copy ()

        if self .bases [i ]=="X":
            meas_circuit .h (0 )# Switch to the X basis before measuring

        meas_circuit .measure ([0 ])
        result =meas_circuit .execute (backend ="simulator",shots =1 )

        # Measurement result
        most_common =max (result .counts ,key =result .counts .get )
        return int (most_common )

    def measure_all (self ,qubits :list [QuantumCircuit ])->list [int ]:
        self .results =[self .measure_qubit (q ,i )for i ,q in enumerate (qubits )]
        return self .results 


class Eve :
    """Eve — eavesdropper (intercept-resend attack)."""

    def __init__ (self ,intercept_rate :float =0.0 ,seed :int =999 ):
        random .seed (seed )
        self .intercept_rate =intercept_rate 
        self .intercepted =0 
        print (f"Eve: taxa de intercepção = {intercept_rate :.0%}")

    def intercept (self ,circuit :QuantumCircuit ,i :int )->QuantumCircuit :
        """
        Eve intercepts, measures randomly, and resends.
        Introduces errors with 25% probability per intercepted qubit.
        """
        if random .random ()>self .intercept_rate :
            return circuit # No interception

        self .intercepted +=1 
        # Eve measures in a random basis
        eve_base =random .choice (BASES )
        meas_circuit =circuit .copy ()

        if eve_base =="X":
            meas_circuit .h (0 )

        meas_circuit .measure ([0 ])
        result =meas_circuit .execute (backend ="simulator",shots =1 )
        eve_bit =int (max (result .counts ,key =result .counts .get ))

        # Re-prepare in the base de Eve
        new_circuit =QuantumCircuit (1 ,name =f"eve_resend_{i }")
        if eve_bit ==1 :
            new_circuit .x (0 )
        if eve_base =="X":
            new_circuit .h (0 )

        return new_circuit 


        # ============================================================================
        # Channel Quantum
        # ============================================================================

class QuantumChannel :
    """Quantum channel with noise and possible eavesdropping."""

    def __init__ (self ,noise_rate :float =0.0 ,eve :Optional [Eve ]=None ):
        self .noise_rate =noise_rate 
        self .eve =eve 

    def transmit (self ,qubits :list [QuantumCircuit ])->list [QuantumCircuit ]:
        """Transmits qubits through the channel, applying noise and/or Eve."""
        received =[]
        for i ,q in enumerate (qubits ):
        # Eavesdropping
            if self .eve :
                q =self .eve .intercept (q ,i )

                # Channel noise (bit-flip com probability noise_rate)
            if self .noise_rate >0 and random .random ()<self .noise_rate :
                noisy =q .copy ()
                noisy .x (0 )# random flip
                q =noisy 

            received .append (q )

        return received 


        # ============================================================================
        # Protocol BB84
        # ============================================================================

def bb84_protocol (
n_bits :int =200 ,
noise_rate :float =0.02 ,
eve_intercept :float =0.0 ,
sifting_sample :float =0.20 ,
)->QKDResult :
    """
    Runs the complete BB84 protocol.

    Args:
        n_bits:         Number of exchanged qubits
        noise_rate:     Channel noise rate (0.0-1.0)
        eve_intercept:  Fraction of qubits intercepted by Eve (0.0=none)
        sifting_sample: Fraction of the key used to estimate QBER

    Returns:
        QKDResult with session statistics
    """

    print (f"\n{'='*55 }")
    print (f"BB84 QKD — {n_bits } qubits | ruído={noise_rate :.0%} | Eve={eve_intercept :.0%}")
    print (f"{'='*55 }")

    # 1. Preparation
    alice =Alice (n_bits ,seed =42 )
    bob =Bob (n_bits ,seed =77 )
    eve =Eve (eve_intercept ,seed =999 )if eve_intercept >0 else None 
    channel =QuantumChannel (noise_rate ,eve )

    # 2. Alice prepairs and sends
    alice_qubits =alice .prepare_all ()
    print (f"Alice enviou {n_bits } qubits")

    # 3. Channel (with possible Eve and noise)
    bob_qubits =channel .transmit (alice_qubits )

    # 4. Bob measures
    bob .measure_all (bob_qubits )

    # 5. Sifting — publicly compare bases (bases only, not bits!)
    sifted_alice =[]
    sifted_bob =[]
    for i in range (n_bits ):
        if alice .bases [i ]==bob .bases [i ]:
            sifted_alice .append (alice .bits [i ])
            sifted_bob .append (bob .results [i ])

    sifted_len =len (sifted_alice )
    print (f"Sifted key: {sifted_len }/{n_bits } bits ({100 *sifted_len /n_bits :.0f}%)")

    # 6. QBER estimation — reveal a public fraction of the key
    n_sample =max (10 ,int (sifted_len *sifting_sample ))
    sample_indices =random .sample (range (sifted_len ),min (n_sample ,sifted_len ))

    errors =sum (sifted_alice [i ]!=sifted_bob [i ]for i in sample_indices )
    qber =errors /len (sample_indices )if sample_indices else 0.0 

    print (f"QBER estimado: {qber :.3f} ({qber :.1%})")

    # 7. Eve detection
    eve_detected =qber >QBER_THRESHOLD 

    if eve_detected :
        print (f"⚠️  Eve detectada! QBER={qber :.1%} > limite={QBER_THRESHOLD :.0%}")
        print ("   Session ABORTED — key compromised")
        final_key =[]
    else :
    # Remove sampling bits from the final key
        remaining =[i for i in range (sifted_len )if i not in sample_indices ]
        final_key =[sifted_alice [i ]for i in remaining ]
        print (f"✅ Canal seguro. Chave final: {len (final_key )} bits")

    efficiency =len (final_key )/n_bits if n_bits >0 else 0.0 

    return QKDResult (
    raw_key_alice =alice .bits [:],
    raw_key_bob =bob .results [:],
    sifted_key =final_key ,
    qber =qber ,
    eve_detected =eve_detected ,
    key_length =len (final_key ),
    efficiency =efficiency ,
    )


    # ============================================================================
    # Demonstration
    # ============================================================================

def print_result (result :QKDResult ,label :str =""):
    print (f"\n{'─'*45 }")
    if label :
        print (f"Cenário: {label }")
    print (f"  Chave final:    {result .key_length } bits")
    print (f"  QBER:           {result .qber :.3f} ({result .qber :.1%})")
    print (f"  Eve detectada:  {'⚠️  YES'if result .eve_detected else '✅ NO'}")
    print (f"  Eficiência:     {result .efficiency :.1%} bits/qubit")
    if result .key_length >0 :
        key_preview =''.join (str (b )for b in result .sifted_key [:32 ])
        print (f"  Chave (32b):    {key_preview }...")


def main ():
    print ("🔐 LightQOS — Demonstration de QKD (Protocol BB84)")
    print ("="*55 )

    # Scenario 1: Channel ideal, sin Eve
    r1 =bb84_protocol (n_bits =300 ,noise_rate =0.0 ,eve_intercept =0.0 )
    print_result (r1 ,"Channel ideal, sin Eve")

    # Scenario 2: Channel com noise, sin Eve
    r2 =bb84_protocol (n_bits =300 ,noise_rate =0.03 ,eve_intercept =0.0 )
    print_result (r2 ,"Channel com 3% noise, sin Eve")

    # Scenario 3: Eve com 50% de interception
    r3 =bb84_protocol (n_bits =300 ,noise_rate =0.01 ,eve_intercept =0.50 )
    print_result (r3 ,"Eve intercepts 50% dos qubits")

    # Scenario 4: Eve com 100% de interception
    r4 =bb84_protocol (n_bits =300 ,noise_rate =0.0 ,eve_intercept =1.00 )
    print_result (r4 ,"Eve intercepts 100% dos qubits")

    print (f"\n\n{'='*55 }")
    print ("Analysis theoretical do QBER vs interception de Eve:")
    print (f"{'─'*45 }")
    print (f"{'Eve (%)':>10}  {'Theoretical QBER':>14}  {'Detectable':>12}")
    for p in [0 ,10 ,25 ,50 ,75 ,100 ]:
    # Theoretical QBER = p/4 * (1 - p/100) ... simplified
        qber_th =(p /100 )*0.25 
        detectable ="✅ Yes"if qber_th >QBER_THRESHOLD else "✅ No"
        print (f"{p :>9}%  {qber_th :>13.3f}  {detectable :>12}")

    print (f"\nLimite seguro QBER < {QBER_THRESHOLD :.0%}")
    print ("✅ QKD demonstration completed!")


if __name__ =="__main__":
    main ()
