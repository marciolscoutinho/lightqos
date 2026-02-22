"""
PennyLane Adapter
"""

try:
    import pennylane as qml
    PENNYLANE_AVAILABLE = True
except ImportError:
    PENNYLANE_AVAILABLE = False

from ..circuit import QuantumCircuit as LightQOSCircuit


def pennylane_to_lightqos(qnode_or_tape) -> LightQOSCircuit:
    """
    Converts a PennyLane QNode or QuantumTape to LightQOS
    """
    if not PENNYLANE_AVAILABLE:
        raise ImportError("PennyLane not installed. Install: pip install pennylane")
    
    # Note: PennyLane uses a different paradigm (functional QNode)
    # This is a simplified implementation
    raise NotImplementedError(
        "The PennyLane adapter requires quantum tape inspection. "
        "Use qml.tape.QuantumTape for conversion."
    )


def lightqos_to_pennylane_ops(lightqos_circuit: LightQOSCircuit):
    """
    Converts a LightQOS circuit into a list of PennyLane operations
    
    Returns:
        List of PennyLane operations that can be used in a QNode
    """
    if not PENNYLANE_AVAILABLE:
        raise ImportError("PennyLane not installed")
    
    ops = []
    
    for op in lightqos_circuit.operations:
        gate = op['gate']
        qubits = op['qubits']
        params = op.get('params', [])
        
        if gate == 'H':
            ops.append(qml.Hadamard(wires=qubits[0]))
        elif gate == 'X':
            ops.append(qml.PauliX(wires=qubits[0]))
        elif gate == 'Y':
            ops.append(qml.PauliY(wires=qubits[0]))
        elif gate == 'Z':
            ops.append(qml.PauliZ(wires=qubits[0]))
        elif gate == 'CNOT':
            ops.append(qml.CNOT(wires=[qubits[0], qubits[1]]))
        elif gate == 'CZ':
            ops.append(qml.CZ(wires=[qubits[0], qubits[1]]))
        elif gate == 'RZ':
            ops.append(qml.RZ(params[0], wires=qubits[0]))
        elif gate == 'RY':
            ops.append(qml.RY(params[0], wires=qubits[0]))
    
    return ops
