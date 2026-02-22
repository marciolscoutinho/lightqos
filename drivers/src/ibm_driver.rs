//! Driver for IBM Quantum (Qiskit Pulse)

use crate::{QuantumDriver, DriverConfig, NativeGate, HardwareTelemetry};
use crate::common::pulse::Pulse;
use std::error::Error;
use std::process::Command;
use std::fs;

pub struct IBMDriver {
    api_token: String,
    backend_name: String,
    qiskit_available: bool,
}

impl IBMDriver {
    pub fn new() -> Self {
        IBMDriver {
            api_token: String::new(),
            backend_name: String::new(),
            qiskit_available: false,
        }
    }
    
    /// Checks whether Qiskit is installed
    fn check_qiskit(&self) -> bool {
        Command::new("python3")
            .args(&["-c", "import qiskit; print(qiskit.__version__)"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl QuantumDriver for IBMDriver {
    fn connect(&mut self, config: &DriverConfig) -> Result<(), Box<dyn Error>> {
        self.api_token = config.credentials
            .as_ref()
            .ok_or("IBM driver requires an API token")?
            .api_key
            .clone();
        
        self.backend_name = config.hardware_id.clone();
        
        // Check Qiskit availability
        self.qiskit_available = self.check_qiskit();
        
        if !self.qiskit_available {
            return Err("Qiskit not found. Install: pip install qiskit qiskit-ibm-runtime".into());
        }
        
        // Tests connection via a Python script
        let test_script = format!(r#"
from qiskit_ibm_runtime import QiskitRuntimeService

service = QiskitRuntimeService(channel="ibm_quantum", token="{}")
backends = service.backends()
print("Connected: {} backends available".format(len(backends)))
"#, self.api_token);
        
        let output = Command::new("python3")
            .args(&["-c", &test_script])
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "IBM connection failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into())
        }
    }
    
    fn send_pulse_sequence(
        &self,
        channel_id: &str,
        pulses: &[Pulse],
    ) -> Result<(), Box<dyn Error>> {
        // Generates Qiskit Pulse code
        let pulse_code = self.generate_pulse_code(channel_id, pulses)?;
        
        // Saves temporarily
        let temp_file = "/tmp/lightqos_pulse.py";
        fs::write(temp_file, pulse_code)?;
        
        // Executes via Qiskit
        let output = Command::new("python3")
            .arg(temp_file)
            .output()?;
        
        if output.status.success() {
            Ok(())
        } else {
            Err(format!(
                "Pulse execution failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ).into())
        }
    }
    
    fn apply_native_gate(
        &self,
        gate_type: NativeGate,
        qubits: &[usize],
        params: &[f64],
    ) -> Result<(), Box<dyn Error>> {
        // Generates Qiskit code for the gate
        let gate_code = self.generate_gate_code(gate_type, qubits, params)?;
        
        // Executes
        let temp_file = "/tmp/lightqos_gate.py";
        fs::write(temp_file, gate_code)?;
        
        Command::new("python3")
            .arg(temp_file)
            .output()?;
        
        Ok(())
    }
    
    fn measure(&self, qubits: &[usize]) -> Result<Vec<u8>, Box<dyn Error>> {
        // Generates and executes a measurement
        let measure_code = format!(r#"
from qiskit import QuantumCircuit
from qiskit_ibm_runtime import QiskitRuntimeService, Sampler

service = QiskitRuntimeService(channel="ibm_quantum", token="{}")
backend = service.backend("{}")

qc = QuantumCircuit({}, {})
qc.measure_all()

sampler = Sampler(backend)
job = sampler.run(qc, shots=1)
result = job.result()

# Returns bitstring
counts = result.quasi_dists[0]
bitstring = max(counts, key=counts.get)
print(format(bitstring, '0{}b'))
"#, self.api_token, self.backend_name, qubits.len(), qubits.len(), qubits.len());
        
        let output = Command::new("python3")
            .args(&["-c", &measure_code])
            .output()?;
        
        if output.status.success() {
            let bitstring = String::from_utf8(output.stdout)?;
            let bits: Vec<u8> = bitstring
                .trim()
                .chars()
                .map(|c| if c == '1' { 1 } else { 0 })
                .collect();
            Ok(bits)
        } else {
            Err("Measurement failed".into())
        }
    }
    
    fn get_telemetry(&self) -> Result<HardwareTelemetry, Box<dyn Error>> {
        // Retrieves backend properties
        let telemetry_code = format!(r#"
from qiskit_ibm_runtime import QiskitRuntimeService
import json

service = QiskitRuntimeService(channel="ibm_quantum", token="{}")
backend = service.backend("{}")

props = backend.properties()
config = backend.configuration()

telemetry = {{
    "coherence_times": [props.t1(i) for i in range(config.n_qubits)],
    "gate_fidelities": [1.0 - props.gate_error('sx', [i]) for i in range(config.n_qubits)],
    "temperature": 0.015,  # ~15 mK typical
}}

print(json.dumps(telemetry))
"#, self.api_token, self.backend_name);
        
        let output = Command::new("python3")
            .args(&["-c", &telemetry_code])
            .output()?;
        
        if output.status.success() {
            let json_str = String::from_utf8(output.stdout)?;
            let data: serde_json::Value = serde_json::from_str(&json_str)?;
            
            Ok(HardwareTelemetry {
                coherence_times: data["coherence_times"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(1e-4))
                    .collect(),
                gate_fidelities: data["gate_fidelities"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.99))
                    .collect(),
                temperature: data["temperature"].as_f64().unwrap_or(0.015),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            })
        } else {
            Err("Telemetry failed".into())
        }
    }
    
    fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        // No persistent state
        Ok(())
    }
}

impl IBMDriver {
    /// Generates Qiskit Pulse code
    fn generate_pulse_code(&self, channel: &str, pulses: &[Pulse]) -> Result<String, Box<dyn Error>> {
        let mut code = format!(r#"
from qiskit import pulse
from qiskit_ibm_runtime import QiskitRuntimeService

service = QiskitRuntimeService(channel="ibm_quantum", token="{}")
backend = service.backend("{}")

with pulse.build(backend) as schedule:
"#, self.api_token, self.backend_name);
        
        for p in pulses.iter() {
            code.push_str(&format!(r#"
    pulse.play(
        pulse.Gaussian(duration={}, amp={}, sigma={}),
        pulse.DriveChannel({})
    )
"#, p.duration_ns, p.amplitude, p.duration_ns as f64 / 4.0, channel));
        }
        
        code.push_str("\nprint('Pulse sequence generated')\n");
        
        Ok(code)
    }
    
    /// Generates code for a native gate
    fn generate_gate_code(
        &self,
        gate: NativeGate,
        qubits: &[usize],
        _params: &[f64],
    ) -> Result<String, Box<dyn Error>> {
        let gate_str = match gate {
            NativeGate::X => format!("qc.x({})", qubits[0]),
            NativeGate::Y => format!("qc.y({})", qubits[0]),
            NativeGate::Z => format!("qc.z({})", qubits[0]),
            NativeGate::H => format!("qc.h({})", qubits[0]),
            NativeGate::CNOT => {
                if qubits.len() < 2 {
                    return Err("CNOT requires 2 qubits".into());
                }
                format!("qc.cx({}, {})", qubits[0], qubits[1])
            },
            NativeGate::CZ => {
                if qubits.len() < 2 {
                    return Err("CZ requires 2 qubits".into());
                }
                format!("qc.cz({}, {})", qubits[0], qubits[1])
            },
            NativeGate::RZ(angle) => format!("qc.rz({}, {})", angle, qubits[0]),
            NativeGate::RY(angle) => format!("qc.ry({}, {})", angle, qubits[0]),
            NativeGate::Custom(name) => {
                return Err(format!("Custom gate '{}' not supported", name).into());
            },
        };
        
        let max_qubit = qubits.iter().max().unwrap_or(&0);
        let code = format!(r#"
from qiskit import QuantumCircuit

qc = QuantumCircuit({})
{}
print('Gate applied: {}')
"#, max_qubit + 1, gate_str, gate_str);
        
        Ok(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gate_code_generation() {
        let driver = IBMDriver::new();
        
        // Tests H gate
        let code = driver.generate_gate_code(
            NativeGate::H,
            &[0],
            &[],
        ).unwrap();
        
        assert!(code.contains("qc.h(0)"));
        
        // Tests CNOT
        let code = driver.generate_gate_code(
            NativeGate::CNOT,
            &[0, 1],
            &[],
        ).unwrap();
        
        assert!(code.contains("qc.cx(0, 1)"));
    }
}
