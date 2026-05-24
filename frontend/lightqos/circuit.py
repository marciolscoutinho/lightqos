# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# circuit.py — QuantumCircuit — high-level quantum circuit builder and executor
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 29-08-2024
# All rights reserved.
# -----------------------------------------------------------------------------

"""
LightQOS Quantum Circuit
"""

from typing import List, Optional, Dict, Any
import numpy as np


class QuantumCircuit:
    """
    Quantum circuit with support for:
    - Temporal Contracts (SLA)
    - QoS specification (Quality of Service)
    - Integration with EFAL/EMF/TLM
    """
    
    def __init__(self, num_qubits: int, name: str = "circuit"):
        self.num_qubits = num_qubits
        self.name = name
        self.operations = []
        self.temporal_contracts = []
        self.qos_contracts = []
        
    def h(self, qubit: int, sla: Optional["TemporalContract"] = None):
        """Hadamard gate with optional SLA"""
        self.operations.append({
            "gate": "H",
            "qubits": [qubit],
            "params": [],
            "sla": sla,
        })
        return self
    
    def cx(self, control: int, target: int, sla: Optional["TemporalContract"] = None):
        """CNOT gate with optional SLA"""
        self.operations.append({
            "gate": "CNOT",
            "qubits": [control, target],
            "params": [],
            "sla": sla,
        })
        return self
    
    def measure(self, qubits: List[int], holographic: bool = False):
        """
        Measurement with HIO, Holographic I/O, option.
        
        Args:
            qubits: List of qubits to measure.
            holographic: If True, uses HIO, Shadow Copies + Multi-Base.
        """
        self.operations.append({
            "gate": "Measure",
            "qubits": qubits,
            "params": [],
            "holographic": holographic,
        })
        return self
    
    def add_temporal_contract(self, contract: "TemporalContract"):
        """Adds a temporal contract to the circuit"""
        self.temporal_contracts.append(contract)
    
    def add_qos_contract(self, contract: "QoSContract"):
        """Adds a QoS contract to the circuit"""
        self.qos_contracts.append(contract)
    
    def to_qasm3(self) -> str:
        """Compiles to OpenQASM 3.0 with LightQOS annotations"""
        qasm = f"""
OPENQASM 3.0;
include "stdgates.inc";

// LightQOS Annotations
// TEMPORAL_CONTRACTS: {len(self.temporal_contracts)}
// QOS_CONTRACTS: {len(self.qos_contracts)}

qubit[{self.num_qubits}] q;
bit[{self.num_qubits}] c;

"""
        
        for op in self.operations:
            gate = op["gate"]
            qubits = op["qubits"]
            
            if gate == "H":
                qasm += f"h q[{qubits[0]}];\n"
            elif gate == "CNOT":
                qasm += f"cx q[{qubits[0]}], q[{qubits[1]}];\n"
            elif gate == "Measure":
                holographic = op.get("holographic", False)
                if holographic:
                    qasm += "// LIGHTQOS_HIO_START\n"
                for q in qubits:
                    qasm += f"c[{q}] = measure q[{q}];\n"
                if holographic:
                    qasm += "// LIGHTQOS_HIO_END\n"
        
        return qasm
    
    def execute(self, backend: str, shots: int = 1024) -> Dict[str, Any]:
        """
        Executes the circuit on the specified backend.
        
        Args:
            backend: Hardware name, such as "ibm_heron", "ionq_forte", etc.
            shots: Number of executions.
            
        Returns:
            Execution results including LightQOS metadata.
        """
        # Communication with the Rust kernel through FFI or subprocess
        import subprocess
        import json
        import tempfile
        
        # Saves a temporary QASM3 file
        with tempfile.NamedTemporaryFile(mode="w", suffix=".qasm", delete=False) as f:
            f.write(self.to_qasm3())
            qasm_path = f.name
        
        # Calls the LightQOS kernel
        result = subprocess.run(
            [
                "lightqos-cli",
                "execute",
                "--circuit",
                qasm_path,
                "--backend",
                backend,
                "--shots",
                str(shots),
            ],
            capture_output=True,
            text=True,
        )
        
        if result.returncode != 0:
            raise RuntimeError(f"Execution failed: {result.stderr}")
        
        return json.loads(result.stdout)

    def x(self, qubit: int, sla: Optional["TemporalContract"] = None):
        """Pauli-X gate"""
        self.operations.append({
            "gate": "X",
            "qubits": [qubit],
            "params": [],
            "sla": sla,
        })
        return self
    
    def y(self, qubit: int, sla: Optional["TemporalContract"] = None):
        """Pauli-Y gate"""
        self.operations.append({
            "gate": "Y",
            "qubits": [qubit],
            "params": [],
            "sla": sla,
        })
        return self
    
    def z(self, qubit: int, sla: Optional["TemporalContract"] = None):
        """Pauli-Z gate"""
        self.operations.append({
            "gate": "Z",
            "qubits": [qubit],
            "params": [],
            "sla": sla,
        })
        return self
    
    def cz(self, control: int, target: int, sla: Optional["TemporalContract"] = None):
        """CZ gate, Controlled-Z"""
        self.operations.append({
            "gate": "CZ",
            "qubits": [control, target],
            "params": [],
            "sla": sla,
        })
        return self
    
    def rz(self, qubit: int, angle: float, sla: Optional["TemporalContract"] = None):
        """Rotation around Z"""
        self.operations.append({
            "gate": "RZ",
            "qubits": [qubit],
            "params": [angle],
            "sla": sla,
        })
        return self
    
    def ry(self, qubit: int, angle: float, sla: Optional["TemporalContract"] = None):
        """Rotation around Y"""
        self.operations.append({
            "gate": "RY",
            "qubits": [qubit],
            "params": [angle],
            "sla": sla,
        })
        return self
    
    def barrier(self):
        """Adds a barrier, synchronization"""
        self.operations.append({
            "gate": "Barrier",
            "qubits": list(range(self.num_qubits)),
            "params": [],
        })
        return self
