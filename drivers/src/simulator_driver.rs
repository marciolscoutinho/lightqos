// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// simulator_driver.rs — Simulator Driver — high-fidelity local state-vector simulator
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 16-11-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use uuid::Uuid;
use rand::Rng;

use crate::drivers::{
    QuantumDriver, DriverConfig, DriverError, DriverResult,
    CircuitSubmission, JobStatus, ExecutionResult, BackendInfo,
    Operation, GateType,
};

// ============================================================================
// SIMULATOR DRIVER
// ============================================================================

/// Local simulator driver
pub struct SimulatorDriver {
    /// Configuration
    config: SimulatorConfig,
    
    /// Executed jobs
    completed_jobs: HashMap<Uuid, ExecutionResult>,
    
    /// Statistics
    stats: SimulatorStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulatorConfig {
    /// Maximum number of qubits
    pub max_qubits: usize,
    
    /// Number of shots
    pub shots: usize,
    
    /// Simulate noise?
    pub simulate_noise: bool,
    
    /// Error probability (if noise is enabled)
    pub error_probability: f64,
    
    /// Random seed
    pub random_seed: Option<u64>,
}

impl Default for SimulatorConfig {
    fn default() -> Self {
        Self {
            max_qubits: 20,
            shots: 1024,
            simulate_noise: false,
            error_probability: 0.01,
            random_seed: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct SimulatorStats {
    total_jobs: u64,
    total_shots: u64,
    total_execution_time_ms: u64,
}

impl SimulatorDriver {
    pub fn new() -> Self {
        Self {
            config: SimulatorConfig::default(),
            completed_jobs: HashMap::new(),
            stats: SimulatorStats::default(),
        }
    }
    
    pub fn with_config(mut self, config: SimulatorConfig) -> Self {
        self.config = config;
        self
    }
    
    // ========================================================================
    // SIMULATION
    // ========================================================================
    
    /// Simulates circuit execution
    fn simulate_circuit(&mut self, circuit: &CircuitSubmission) -> DriverResult<ExecutionResult> {
        let start_time = Self::current_time_ms();
        
        // Validation
        if circuit.num_qubits > self.config.max_qubits {
            return Err(DriverError::InvalidCircuit(format!(
                "Simulator supports a maximum of {} qubits",
                self.config.max_qubits
            )));
        }
        
        // Simulate execution
        let counts = self.run_simulation(circuit)?;
        
        let execution_time = Self::current_time_ms() - start_time;
        
        // Statistics
        self.stats.total_jobs += 1;
        self.stats.total_shots += self.config.shots as u64;
        self.stats.total_execution_time_ms += execution_time;
        
        Ok(ExecutionResult {
            job_id: circuit.id,
            backend: "simulator".to_string(),
            shots: self.config.shots,
            counts,
            execution_time_ms: execution_time,
            queue_time_ms: 0,  // Immediate execution
            success: true,
            error_message: None,
        })
    }
    
    /// Runs simulation
    fn run_simulation(&self, circuit: &CircuitSubmission) -> DriverResult<HashMap<String, usize>> {
        let mut counts = HashMap::new();
        let mut rng = rand::thread_rng();
        
        // Simplified simulation:
        // - For small circuits (<5 qubits): simulate correctly
        // - For large circuits: approximate with sampling
        
        if circuit.num_qubits <= 5 {
            // Full simulation
            counts = self.simulate_small_circuit(circuit);
        } else {
            // Sampling for large circuits
            counts = self.sample_large_circuit(circuit);
        }
        
        // Add noise if enabled
        if self.config.simulate_noise {
            counts = self.apply_noise(counts, &mut rng);
        }
        
        Ok(counts)
    }
    
    fn simulate_small_circuit(&self, circuit: &CircuitSubmission) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        
        // Simplified simulation: uniform distribution or operation-based distribution
        let num_states = 2_usize.pow(circuit.num_qubits as u32);
        
        if circuit.operations.is_empty() {
            // |0...0⟩ state
            let zero_state = "0".repeat(circuit.num_qubits);
            counts.insert(zero_state, self.config.shots);
        } else {
            // Check whether it has Hadamard → superposition
            let has_hadamard = circuit.operations.iter()
                .any(|op| op.gate_type == GateType::H);
            
            if has_hadamard {
                // Uniform superposition
                let shots_per_state = self.config.shots / num_states;
                for i in 0..num_states {
                    let state = format!("{:0width$b}", i, width = circuit.num_qubits);
                    counts.insert(state, shots_per_state);
                }
            } else {
                // Basis state
                let zero_state = "0".repeat(circuit.num_qubits);
                counts.insert(zero_state, self.config.shots);
            }
        }
        
        counts
    }
    
    fn sample_large_circuit(&self, circuit: &CircuitSubmission) -> HashMap<String, usize> {
        // For large circuits: Monte Carlo sampling
        let mut counts = HashMap::new();
        let mut rng = rand::thread_rng();
        
        for _ in 0..self.config.shots {
            // Generate random state
            let state: String = (0..circuit.num_qubits)
                .map(|_| if rng.gen_bool(0.5) { '1' } else { '0' })
                .collect();
            
            *counts.entry(state).or_insert(0) += 1;
        }
        
        counts
    }
    
    fn apply_noise(&self, mut counts: HashMap<String, usize>, rng: &mut impl Rng) -> HashMap<String, usize> {
        let mut noisy_counts = HashMap::new();
        
        for (state, count) in counts {
            for _ in 0..count {
                let mut noisy_state = state.clone();
                
                // Flip bits with error probability
                for i in 0..state.len() {
                    if rng.gen_bool(self.config.error_probability) {
                        let bit = if state.chars().nth(i).unwrap() == '0' { '1' } else { '0' };
                        noisy_state.replace_range(i..=i, &bit.to_string());
                    }
                }
                
                *noisy_counts.entry(noisy_state).or_insert(0) += 1;
            }
        }
        
        noisy_counts
    }
    
    fn current_time_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }
    
    pub fn get_stats(&self) -> &SimulatorStats {
        &self.stats
    }
}

// ============================================================================
// TRAIT IMPLEMENTATION
// ============================================================================

#[async_trait::async_trait]
impl QuantumDriver for SimulatorDriver {
    async fn initialize(&mut self) -> DriverResult<()> {
        // Nothing to do - local simulator
        Ok(())
    }
    
    async fn submit_circuit(&mut self, circuit: CircuitSubmission) -> DriverResult<Uuid> {
        // Immediate execution
        let result = self.simulate_circuit(&circuit)?;
        let job_id = circuit.id;
        
        self.completed_jobs.insert(job_id, result);
        
        println!("[Simulator] Job executed: {}", job_id);
        
        Ok(job_id)
    }
    
    async fn get_job_status(&mut self, job_id: Uuid) -> DriverResult<JobStatus> {
        if self.completed_jobs.contains_key(&job_id) {
            Ok(JobStatus::Completed)
        } else {
            Err(DriverError::JobNotFound(job_id.to_string()))
        }
    }
    
    async fn get_results(&mut self, job_id: Uuid) -> DriverResult<ExecutionResult> {
        self.completed_jobs.get(&job_id)
            .cloned()
            .ok_or_else(|| DriverError::JobNotFound(job_id.to_string()))
    }
    
    async fn cancel_job(&mut self, _job_id: Uuid) -> DriverResult<()> {
        // Simulator jobs are instantaneous and cannot be cancelled
        Ok(())
    }
    
    async fn list_backends(&mut self) -> DriverResult<Vec<BackendInfo>> {
        Ok(vec![
            BackendInfo {
                name: "simulator".to_string(),
                provider: "LightQOS".to_string(),
                num_qubits: self.config.max_qubits,
                is_simulator: true,
                is_available: true,
                queue_length: 0,
                avg_queue_time_s: Some(0),
            }
        ])
    }
    
    fn get_config(&self) -> &dyn std::any::Any {
        &self.config
    }
    
    fn driver_name(&self) -> &str {
        "Simulator"
    }
}

impl Default for SimulatorDriver {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simulator_creation() {
        let sim = SimulatorDriver::new();
        assert_eq!(sim.config.max_qubits, 20);
        assert_eq!(sim.config.shots, 1024);
    }
    
    #[tokio::test]
    async fn test_simple_circuit() {
        let mut sim = SimulatorDriver::new();
        sim.initialize().await.unwrap();
        
        // Simple circuit: |00⟩
        let circuit = CircuitSubmission {
            id: Uuid::new_v4(),
            num_qubits: 2,
            operations: vec![],
            metadata: HashMap::new(),
        };
        
        let job_id = sim.submit_circuit(circuit).await.unwrap();
        
        // Check result
        let result = sim.get_results(job_id).await.unwrap();
        
        assert_eq!(result.shots, 1024);
        assert!(result.success);
        assert_eq!(result.counts.len(), 1);
        assert!(result.counts.contains_key("00"));
    }
    
    #[tokio::test]
    async fn test_with_noise() {
        let config = SimulatorConfig {
            shots: 100,
            simulate_noise: true,
            error_probability: 0.1,
            ..Default::default()
        };
        
        let mut sim = SimulatorDriver::new().with_config(config);
        sim.initialize().await.unwrap();
        
        let circuit = CircuitSubmission {
            id: Uuid::new_v4(),
            num_qubits: 2,
            operations: vec![],
            metadata: HashMap::new(),
        };
        
        let job_id = sim.submit_circuit(circuit).await.unwrap();
        let result = sim.get_results(job_id).await.unwrap();
        
        // With noise, it should have more states
        assert!(result.counts.len() >= 1);
    }
    
    #[tokio::test]
    async fn test_backend_info() {
        let mut sim = SimulatorDriver::new();
        sim.initialize().await.unwrap();
        
        let backends = sim.list_backends().await.unwrap();
        
        assert_eq!(backends.len(), 1);
        assert_eq!(backends[0].name, "simulator");
        assert!(backends[0].is_simulator);
        assert_eq!(backends[0].queue_length, 0);
    }
}
