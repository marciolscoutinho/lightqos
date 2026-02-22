"""
Cirq Adapter
"""

try:
    import cirq
    CIRQ_AVAILABLE = True
except ImportError:
    CIRQ_AVAILABLE = False

from ..circuit import QuantumCircuit as LightQOSCircuit


def cirq_to_lightqos(cirq_circuit: 'cirq.Circuit') -> LightQOSCircuit:
    """
    Converts a Cirq circuit to LightQOS
    """
    if not CIRQ_AVAILABLE:
        raise ImportError("Cirq not installed. Install: pip install cirq")
    
    # Get qubits
    qubits_list = sorted(cirq_circuit.all_qubits())
    num_qubits = len(qubits_list)
    qubit_map = {q: i for i, q in enumerate(qubits_list)}
    
    lightqos_circuit = LightQOSCircuit(num_qubits)
    
    for moment in cirq_circuit:
        for op in moment:
            gate = op.gate
            qubit_indices = [qubit_map[q] for q in op.qubits]
            
            if isinstance(gate, cirq.H):
                lightqos_circuit.h(qubit_indices[0])
            elif isinstance(gate, cirq.X):
                lightqos_circuit.x(qubit_indices[0])
            elif isinstance(gate, cirq.Y):
                lightqos_circuit.y(qubit_indices[0])
            elif isinstance(gate, cirq.Z):
                lightqos_circuit.z(qubit_indices[0])
            elif isinstance(gate, cirq.CNOT):
                lightqos_circuit.cx(qubit_indices[0], qubit_indices[1])
            elif isinstance(gate, cirq.CZ):
                lightqos_circuit.cz(qubit_indices[0], qubit_indices[1])
            elif isinstance(gate, cirq.Rz):
                lightqos_circuit.rz(qubit_indices[0], gate.exponent * 3.14159)
            elif isinstance(gate, cirq.Ry):
                lightqos_circuit.ry(qubit_indices[0], gate.exponent * 3.14159)
    
    return lightqos_circuit


def lightqos_to_cirq(lightqos_circuit: LightQOSCircuit) -> 'cirq.Circuit':
    """
    Converts a LightQOS circuit to Cirq
    """
    if not CIRQ_AVAILABLE:
        raise ImportError("Cirq not installed")
    
    qubits = [cirq.LineQubit(i) for i in range(lightqos_circuit.num_qubits)]
    circuit = cirq.Circuit()
    
    for op in lightqos_circuit.operations:
        gate = op['gate']
        qubit_indices = op['qubits']
        params = op.get('params', [])
        
        if gate == 'H':
            circuit.append(cirq.H(qubits[qubit_indices[0]]))
        elif gate == 'X':
            circuit.append(cirq.X(qubits[qubit_indices[0]]))
        elif gate == 'Y':
            circuit.append(cirq.Y(qubits[qubit_indices[0]]))
        elif gate == 'Z':
            circuit.append(cirq.Z(qubits[qubit_indices[0]]))
        elif gate == 'CNOT':
            circuit.append(cirq.CNOT(qubits[qubit_indices[0]], qubits[qubit_indices[1]]))
        elif gate == 'CZ':
            circuit.append(cirq.CZ(qubits[qubit_indices[0]], qubits[qubit_indices[1]]))
        elif gate == 'RZ':
            circuit.append(cirq.Rz(rads=params[0])(qubits[qubit_indices[0]]))
        elif gate == 'RY':
            circuit.append(cirq.Ry(rads=params[0])(qubits[qubit_indices[0]]))
        elif gate == 'Measure':
            for q in qubit_indices:
                circuit.append(cirq.measure(qubits[q]))
    
    return circuit
