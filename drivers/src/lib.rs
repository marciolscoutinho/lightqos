//! Hardware Drivers for LightQOS
//! 
//! Common interface for communicating with different quantum platforms

pub mod qblox_driver;
pub mod ionq_driver;
pub mod zurich_driver;
pub mod ibm_driver;
pub mod photonic_driver;
pub mod common;

use std::error::Error;

/// Common interface for all drivers
pub trait QuantumDriver {
    /// Initializes the connection to the hardware
    fn connect(&mut self, config: &DriverConfig) -> Result<(), Box<dyn Error>>;
    
    /// Sends a pulse sequence (EFAL → Hardware)
    fn send_pulse_sequence(
        &self,
        channel_id: &str,
        pulses: &[common::pulse::Pulse],
    ) -> Result<(), Box<dyn Error>>;
    
    /// Applies a native quantum gate
    fn apply_native_gate(
        &self,
        gate_type: NativeGate,
        qubits: &[usize],
        params: &[f64],
    ) -> Result<(), Box<dyn Error>>;
    
    /// Performs measurement
    fn measure(&self, qubits: &[usize]) -> Result<Vec<usize>, Box<dyn Error>>;
    
    /// Retrieves hardware telemetry
    fn get_telemetry(&self) -> Result<HardwareTelemetry, Box<dyn Error>>;
    
    /// Closes the connection
    fn disconnect(&mut self) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
pub struct DriverConfig {
    pub platform: String,
    pub api_endpoint: Option<String>,
    pub credentials: Option<Credentials>,
    pub hardware_id: String,
}

#[derive(Clone)]
pub struct Credentials {
    pub api_key: String,
    pub secret: Option<String>,
}

pub enum NativeGate {
    X,
    Y,
    Z,
    H,
    CNOT,
    CZ,
    RZ(f64),
    RY(f64),
    Custom(String),
}

pub struct HardwareTelemetry {
    pub coherence_times: Vec<f64>, // T1, T2 per qubit (simplified)
    pub gate_fidelities: Vec<f64>,
    pub temperature: f64,
    pub timestamp: u64,
}
