"""
Qiskit Adapter
"""

from typing import Optional
try:
    from qiskit import QuantumCircuit as QiskitCircuit
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False

from ..circuit import QuantumCircuit as LightQOSCircuit


def qiskit_to_lightqos(qiskit_circuit: 'QiskitCircuit') -> LightQOSCircuit:
    """
    Converts a Qiskit circuit to LightQOS
    
    Args:
        qiskit_circuit: Qiskit circuit
        
    Returns:
        Equivalent LightQOS circuit
    """
    if not QISKIT_AVAILABLE:
        raise ImportError("Qiskit not installed. Install: pip install qiskit")
    
    # Creates a LightQOS circuit
    num_qubits = qiskit_circuit.num_qubits
    lightqos_circuit = LightQOSCircuit(num_qubits, name=qiskit_circuit.name)
    
    # Converts operations
    for instruction, qargs, cargs in qiskit_circuit.data:
        gate_name = instruction.name
        qubit_indices = [qiskit_circuit.find_bit(q).index for q in qargs]
        params = instruction.params
        
        # Maps gates
        if gate_name == 'h':
            lightqos_circuit.h(qubit_indices[0])
        elif gate_name == 'x':
            lightqos_circuit.x(qubit_indices[0])
        elif gate_name == 'y':
            lightqos_circuit.y(qubit_indices[0])
        elif gate_name == 'z':
            lightqos_circuit.z(qubit_indices[0])
        elif gate_name == 'cx':
            lightqos_circuit.cx(qubit_indices[0], qubit_indices[1])
        elif gate_name == 'cz':
            lightqos_circuit.cz(qubit_indices[0], qubit_indices[1])
        elif gate_name == 'rz':
            lightqos_circuit.rz(qubit_indices[0], params[0])
        elif gate_name == 'ry':
            lightqos_circuit.ry(qubit_indices[0], params[0])
        elif gate_name == 'measure':
            # Measurements are handled separately
            pass
        else:
            print(f"Warning: Gate '{gate_name}' not supported, skipping")
    
    return lightqos_circuit


def lightqos_to_qiskit(lightqos_circuit: LightQOSCircuit) -> 'QiskitCircuit':
    """
    Converts a LightQOS circuit to Qiskit
    
    Args:
        lightqos_circuit: LightQOS circuit
        
    Returns:
        Equivalent Qiskit circuit
    """
    if not QISKIT_AVAILABLE:
        raise ImportError("Qiskit not installed")
    
    qiskit_circuit = QiskitCircuit(lightqos_circuit.num_qubits)
    
    for op in lightqos_circuit.operations:
        gate = op['gate']
        qubits = op['qubits']
        params = op.get('params', [])
        
        if gate == 'H':
            qiskit_circuit.h(qubits[0])
        elif gate == 'X':
            qiskit_circuit.x(qubits[0])
        elif gate == 'Y':
            qiskit_circuit.y(qubits[0])
        elif gate == 'Z':
            qiskit_circuit.z(qubits[0])
        elif gate == 'CNOT':
            qiskit_circuit.cx(qubits[0], qubits[1])
        elif gate == 'CZ':
            qiskit_circuit.cz(qubits[0], qubits[1])
        elif gate == 'RZ':
            qiskit_circuit.rz(params[0], qubits[0])
        elif gate == 'RY':
            qiskit_circuit.ry(params[0], qubits[0])
        elif gate == 'Measure':
            for q in qubits:
                qiskit_circuit.measure(q, q)
    
    return qiskit_circuit
