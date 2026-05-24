# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# cirq_adapter.py — Cirq Adapter — bidirectional Cirq ↔ LightQOS conversion
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 06-08-2025
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Adapter for Cirq
"""

try:
    import cirq
    CIRQ_AVAILABLE = True
except ImportError:
    CIRQ_AVAILABLE = False

from ..circuit import QuantumCircuit as LightQOSCircuit


def cirq_to_lightqos(cirq_circuit: "cirq.Circuit") -> LightQOSCircuit:
    """
    Converts a Cirq circuit into a LightQOS circuit
    """
    if not CIRQ_AVAILABLE:
        raise ImportError("Cirq not installed. Install: pip install cirq")
    
    # Gets qubits
    qubits_list = sorted(cirq_circuit.all_qubits())
    num_qubits = len(qubits_list)
    qubit_map = {q: i for i, q in enumerate(qubits_list)}
    
    lightqos_circuit = LightQOSCircuit(num_qubits)
    
    for moment in cirq_circuit:
        for op in moment:
            gate = op.gate
            qubit_indices = [qubit_map[q] for q in op.qubits]
            
            if gate == cirq.H:
                lightqos_circuit.h(qubit_indices[0])
            elif gate == cirq.X:
                lightqos_circuit.x(qubit_indices[0])
            elif gate == cirq.Y:
                lightqos_circuit.y(qubit_indices[0])
            elif gate == cirq.Z:
                lightqos_circuit.z(qubit_indices[0])
            elif gate == cirq.CNOT:
                lightqos_circuit.cx(qubit_indices[0], qubit_indices[1])
            elif gate == cirq.CZ:
                lightqos_circuit.cz(qubit_indices[0], qubit_indices[1])
            elif isinstance(gate, cirq.ZPowGate):
                lightqos_circuit.rz(qubit_indices[0], gate.exponent * 3.14159)
            elif isinstance(gate, cirq.YPowGate):
                lightqos_circuit.ry(qubit_indices[0], gate.exponent * 3.14159)
    
    return lightqos_circuit


def lightqos_to_cirq(lightqos_circuit: LightQOSCircuit) -> "cirq.Circuit":
    """
    Converts a LightQOS circuit into a Cirq circuit
    """
    if not CIRQ_AVAILABLE:
        raise ImportError("Cirq not installed")
    
    qubits = [cirq.LineQubit(i) for i in range(lightqos_circuit.num_qubits)]
    circuit = cirq.Circuit()
    
    for op in lightqos_circuit.operations:
        gate = op["gate"]
        qubit_indices = op["qubits"]
        params = op.get("params", [])
        
        if gate == "H":
            circuit.append(cirq.H(qubits[qubit_indices[0]]))
        elif gate == "X":
            circuit.append(cirq.X(qubits[qubit_indices[0]]))
        elif gate == "Y":
            circuit.append(cirq.Y(qubits[qubit_indices[0]]))
        elif gate == "Z":
            circuit.append(cirq.Z(qubits[qubit_indices[0]]))
        elif gate == "CNOT":
            circuit.append(cirq.CNOT(qubits[qubit_indices[0]], qubits[qubit_indices[1]]))
        elif gate == "CZ":
            circuit.append(cirq.CZ(qubits[qubit_indices[0]], qubits[qubit_indices[1]]))
        elif gate == "RZ":
            circuit.append(cirq.rz(rads=params[0])(qubits[qubit_indices[0]]))
        elif gate == "RY":
            circuit.append(cirq.ry(rads=params[0])(qubits[qubit_indices[0]]))
        elif gate == "Measure":
            for q in qubit_indices:
                circuit.append(cirq.measure(qubits[q]))
    
    return circuit
