// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — Drivers module — unified QuantumDriver trait and hardware abstraction
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 15-01-2022
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod ibm_driver;
pub mod ionq_driver;
pub mod simulator_driver;
pub mod qblox_driver;
pub mod zurich_driver;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// ============================================================================
// TRAIT QUANTUM DRIVER
// ============================================================================

/// Unified interface for quantum drivers
#[async_trait::async_trait]
pub trait QuantumDriver: Send + Sync {
    /// Initializes driver (authentication, connection)
    async fn initialize(&mut self) -> DriverResult<()>;
    
    /// Submits circuit for execution
    async fn submit_circuit(&mut self, circuit: CircuitSubmission) -> DriverResult<Uuid>;
    
    /// Queries job status
    async fn get_job_status(&mut self, job_id: Uuid) -> DriverResult<JobStatus>;
    
    /// Gets execution results
    async fn get_results(&mut self, job_id: Uuid) -> DriverResult<ExecutionResult>;
    
    /// Cancels running job
    async fn cancel_job(&mut self, job_id: Uuid) -> DriverResult<()>;
    
    /// Lists available backends
    async fn list_backends(&mut self) -> DriverResult<Vec<BackendInfo>>;
    
    /// Returns configuration
    fn get_config(&self) -> &dyn std::any::Any;
    
    /// Driver name
    fn driver_name(&self) -> &str;
}

// ============================================================================
// COMMON STRUCTURES
// ============================================================================

/// Circuit submission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitSubmission {
    /// Unique ID
    pub id: Uuid,
    
    /// Number of qubits
    pub num_qubits: usize,
    
    /// Circuit operations (internal IR)
    pub operations: Vec<Operation>,
    
    /// Metadata
    pub metadata: HashMap<String, String>,
}

/// Quantum operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub gate_type: GateType,
    pub qubits: Vec<usize>,
    pub parameters: Vec<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    H,      // Hadamard
    X,      // Pauli-X
    Y,      // Pauli-Y
    Z,      // Pauli-Z
    CNOT,   // Controlled-NOT
    CZ,     // Controlled-Z
    RX,     // X rotation
    RY,     // Y rotation
    RZ,     // Z rotation
    T,      // T gate
    S,      // S gate
    Measure,
}

/// Job status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,       // Queued
    Running,      // Running
    Completed,    // Completed
    Failed,       // Failed
    Cancelled,    // Cancelled
}

/// Execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Job ID
    pub job_id: Uuid,
    
    /// Backend usado
    pub backend: String,
    
    /// Number of shots
    pub shots: usize,
    
    /// Measurement counts
    pub counts: HashMap<String, usize>,
    
    /// Execution time (ms)
    pub execution_time_ms: u64,
    
    /// Queue time (ms)
    pub queue_time_ms: u64,
    
    /// Success?
    pub success: bool,
    
    /// Error message (if failure)
    pub error_message: Option<String>,
}

impl ExecutionResult {
    /// Calculates probabilities from counts
    pub fn probabilities(&self) -> HashMap<String, f64> {
        let total = self.shots as f64;
        
        self.counts.iter()
            .map(|(state, count)| {
                (state.clone(), *count as f64 / total)
            })
            .collect()
    }
    
    /// Returns the most likely state
    pub fn most_probable_state(&self) -> Option<(String, f64)> {
        self.probabilities()
            .into_iter()
            .max_by(|(_, p1), (_, p2)| p1.partial_cmp(p2).unwrap())
    }
}

/// Backend information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendInfo {
    /// Backend name
    pub name: String,
    
    /// Provider (IBM, IonQ, etc.)
    pub provider: String,
    
    /// Number of qubits
    pub num_qubits: usize,
    
    /// Is simulator?
    pub is_simulator: bool,
    
    /// Is available?
    pub is_available: bool,
    
    /// Queue length
    pub queue_length: usize,
    
    /// Average queue time (seconds)
    pub avg_queue_time_s: Option<u64>,
}

/// Driver configuration
pub trait DriverConfig: Send + Sync {
    fn shots(&self) -> usize;
    fn timeout_s(&self) -> u64;
}

// ============================================================================
// ERRORS
// ============================================================================

/// Driver errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum DriverError {
    #[error("Driver not initialized")]
    NotInitialized,
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Invalid circuit: {0}")]
    InvalidCircuit(String),
    
    #[error("Job not found: {0}")]
    JobNotFound(String),
    
    #[error("Job is not ready")]
    JobNotReady,
    
    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),
    
    #[error("Timeout while waiting for job")]
    Timeout,
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Provider error: {0}")]
    ProviderError(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type DriverResult<T> = Result<T, DriverError>;

// ============================================================================
// DRIVER MANAGER
// ============================================================================

/// Multiple-driver manager
pub struct DriverManager {
    drivers: HashMap<String, Box<dyn QuantumDriver>>,
    active_driver: Option<String>,
}

impl DriverManager {
    pub fn new() -> Self {
        Self {
            drivers: HashMap::new(),
            active_driver: None,
        }
    }
    
    /// Registers driver
    pub fn register_driver(&mut self, name: String, driver: Box<dyn QuantumDriver>) {
        self.drivers.insert(name, driver);
    }
    
    /// Sets active driver
    pub fn set_active(&mut self, name: String) -> DriverResult<()> {
        if !self.drivers.contains_key(&name) {
            return Err(DriverError::Unknown(format!("Driver '{}' not found", name)));
        }
        
        self.active_driver = Some(name);
        Ok(())
    }
    
    /// Gets active driver
    pub fn get_active(&mut self) -> Option<&mut Box<dyn QuantumDriver>> {
        if let Some(ref name) = self.active_driver {
            self.drivers.get_mut(name)
        } else {
            None
        }
    }
    
    /// Gets driver by name
    pub fn get_driver(&mut self, name: &str) -> Option<&mut Box<dyn QuantumDriver>> {
        self.drivers.get_mut(name)
    }
    
    /// Lists available drivers
    pub fn list_drivers(&self) -> Vec<String> {
        self.drivers.keys().cloned().collect()
    }
}

impl Default for DriverManager {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export drivers
pub use ibm_driver::{IBMQuantumDriver, IBMConfig, IBMTranspiler};
pub use ionq_driver::{IonQDriver, IonQConfig};
pub use simulator_driver::{SimulatorDriver, SimulatorConfig};
pub use qblox_driver::{QbloxCluster, QbloxModule, QbloxPulse, QbloxModuleType, PulseEnvelope};
pub use zurich_driver::{ZurichSetup, ZIInstrument, ZIPulse, ZIInstrumentType};
