// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// process_tensor.rs — Process Tensor — multi-time quantum process representation
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 11-02-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::quantum_channel::QuantumChannel;
use crate::memory_kernel::MemoryKernel;
use nalgebra::DMatrix;
use num_complex::Complex64;
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// PROCESS TENSOR
// ============================================================================

/// Process tensor - describes quantum process with memory
#[derive(Debug, Clone)]
pub struct ProcessTensor {
    /// Unique identifier
    pub id: Uuid,
    
    /// Temporal steps
    pub steps: Vec<TemporalStep>,
    
    /// Memory kernel (if non-Markovian)
    pub memory: Option<MemoryKernel>,
    
    /// System dimension
    pub system_dim: usize,
    
    /// Total number of time steps
    pub num_steps: usize,
    
    /// Is process Markovian?
    pub is_markovian: bool,
}

impl ProcessTensor {
    /// Create new process tensor
    pub fn new(system_dim: usize, num_steps: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            steps: Vec::new(),
            memory: None,
            system_dim,
            num_steps,
            is_markovian: true,
        }
    }
    
    /// Add temporal step
    pub fn add_step(&mut self, step: TemporalStep) {
        self.steps.push(step);
    }
    
    /// Set memory kernel (makes process non-Markovian)
    pub fn set_memory(&mut self, memory: MemoryKernel) {
        self.memory = Some(memory);
        self.is_markovian = false;
    }
    
    /// Apply process to initial state
    pub fn apply(&self, initial_state: &DMatrix<Complex64>) -> MultiTimeProcess {
        let mut process = MultiTimeProcess::new(self.system_dim);
        process.add_state(0, initial_state.clone());
        
        let mut current_state = initial_state.clone();
        
        for (idx, step) in self.steps.iter().enumerate() {
            // Apply channel
            current_state = step.channel.apply(&current_state);
            
            // Apply memory effects if non-Markovian
            if let Some(ref memory) = self.memory {
                current_state = memory.apply_memory_effect(
                    &current_state,
                    step.time,
                    &process.states,
                );
            }
            
            process.add_state(step.time, current_state.clone());
        }
        
        process
    }
    
    /// Compute process matrix (tensor representation)
    pub fn compute_process_matrix(&self) -> DMatrix<Complex64> {
        // Process matrix Υ encodes all possible interventions
        let dim = self.system_dim.pow(self.num_steps as u32);
        let mut upsilon = DMatrix::zeros(dim, dim);
        
        // Build process matrix from channels
        // Υ = Σ_{i₁,...,iₙ} |i₁⟩⟨i₁| ⊗ ... ⊗ |iₙ⟩⟨iₙ| ⊗ Ε_{i₁,...,iₙ}
        
        // Simplified implementation
        upsilon[(0, 0)] = Complex64::new(1.0, 0.0);
        
        upsilon
    }
    
    /// Check if process satisfies causality
    pub fn is_causal(&self) -> bool {
        // Check temporal ordering
        for i in 0..self.steps.len() - 1 {
            if self.steps[i].time >= self.steps[i + 1].time {
                return false;
            }
        }
        true
    }
    
    /// Extract channel at specific time step
    pub fn get_channel(&self, step_index: usize) -> Option<&QuantumChannel> {
        self.steps.get(step_index).map(|step| &step.channel)
    }
    
    /// Compute non-Markovianity measure
    pub fn non_markovianity_measure(&self) -> f64 {
        if self.is_markovian {
            return 0.0;
        }
        
        // Simplified BLP measure (Breuer-Laine-Piilo)
        // Current implementation would track trace distance increases
        if let Some(ref memory) = self.memory {
            memory.memory_strength()
        } else {
            0.0
        }
    }
}

// ============================================================================
// TEMPORAL STEP
// ============================================================================

/// Single temporal step in process
#[derive(Debug, Clone)]
pub struct TemporalStep {
    /// Time of this step
    pub time: f64,
    
    /// Quantum channel applied at this time
    pub channel: QuantumChannel,
    
    /// Intervention (if any)
    pub intervention: Option<Intervention>,
}

impl TemporalStep {
    /// Create new temporal step
    pub fn new(time: f64, channel: QuantumChannel) -> Self {
        Self {
            time,
            channel,
            intervention: None,
        }
    }
    
    /// Add intervention
    pub fn with_intervention(mut self, intervention: Intervention) -> Self {
        self.intervention = Some(intervention);
        self
    }
}

// ============================================================================
// INTERVENTION
// ============================================================================

/// Quantum intervention at a time step
#[derive(Debug, Clone)]
pub struct Intervention {
    /// Type of intervention
    pub intervention_type: InterventionType,
    
    /// Parameters
    pub parameters: HashMap<String, f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterventionType {
    /// No intervention (identity)
    None,
    
    /// Measurement
    Measurement,
    
    /// Unitary gate
    Unitary,
    
    /// Feedback control
    Feedback,
}

// ============================================================================
// MULTI-TIME PROCESS
// ============================================================================

/// Multi-time quantum process result
#[derive(Debug, Clone)]
pub struct MultiTimeProcess {
    /// System dimension
    pub system_dim: usize,
    
    /// States at different times
    pub states: HashMap<usize, DMatrix<Complex64>>,
    
    /// Time steps
    pub times: Vec<f64>,
}

impl MultiTimeProcess {
    /// Create new multi-time process
    pub fn new(system_dim: usize) -> Self {
        Self {
            system_dim,
            states: HashMap::new(),
            times: Vec::new(),
        }
    }
    
    /// Add state at time
    pub fn add_state(&mut self, time_index: usize, state: DMatrix<Complex64>) {
        self.states.insert(time_index, state);
    }
    
    /// Get state at time
    pub fn get_state(&self, time_index: usize) -> Option<&DMatrix<Complex64>> {
        self.states.get(&time_index)
    }
    
    /// Compute fidelity between two time points
    pub fn fidelity(&self, time1: usize, time2: usize) -> Option<f64> {
        let state1 = self.states.get(&time1)?;
        let state2 = self.states.get(&time2)?;
        
        // F = Tr(√(√ρ₁ ρ₂ √ρ₁))²
        // Simplified for pure states: F = |⟨ψ₁|ψ₂⟩|²
        let trace = state1.component_mul(state2).sum();
        Some(trace.norm_sqr())
    }
    
    /// Export to time series data
    pub fn to_time_series(&self) -> Vec<(usize, Vec<Complex64>)> {
        let mut series = Vec::new();
        
        for (time, state) in &self.states {
            let state_vec: Vec<Complex64> = state.iter().copied().collect();
            series.push((*time, state_vec));
        }
        
        series.sort_by_key(|(t, _)| *t);
        series
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_tensor_creation() {
        let pt = ProcessTensor::new(2, 3);
        assert_eq!(pt.system_dim, 2);
        assert_eq!(pt.num_steps, 3);
        assert!(pt.is_markovian);
    }
    
    #[test]
    fn test_add_temporal_step() {
        let mut pt = ProcessTensor::new(2, 2);
        
        let channel1 = QuantumChannel::identity(2);
        let step1 = TemporalStep::new(0.0, channel1);
        pt.add_step(step1);
        
        let channel2 = QuantumChannel::depolarizing(2, 0.1);
        let step2 = TemporalStep::new(1.0, channel2);
        pt.add_step(step2);
        
        assert_eq!(pt.steps.len(), 2);
        assert!(pt.is_causal());
    }
    
    #[test]
    fn test_multi_time_process() {
        let mut process = MultiTimeProcess::new(2);
        
        let state0 = DMatrix::identity(2, 2);
        process.add_state(0, state0.clone());
        
        let state1 = DMatrix::identity(2, 2) * 0.9;
        process.add_state(1, state1);
        
        assert_eq!(process.states.len(), 2);
        assert!(process.get_state(0).is_some());
    }
}
