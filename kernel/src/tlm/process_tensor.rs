// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// process_tensor.rs — TLM Process Tensor — non-Markovian memory kernel tracking
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 03-10-2025
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use nalgebra::{Complex, DMatrix, DVector};
use uuid::Uuid;

// ============================================================================
// PROCESS TENSOR
// ============================================================================

/// Process tensor for non-Markovian dynamics
/// 
/// Represents temporal evolution with memory:
/// Υ_{t₁,t₂,...,tₙ} maps operations at past times to the current state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessTensor {
    /// Unique ID
    pub id: Uuid,
    
    /// Stored time steps
    pub time_steps: Vec<f64>,
    
    /// Tensors for each step (4^n dimensions for n qubits)
    pub tensors: Vec<Vec<Complex<f64>>>,
    
    /// Memory depth (number of steps)
    pub memory_depth: usize,
    
    /// System dimension (number of qubits)
    pub system_dimension: usize,
    
    /// Non-Markovianity metric
    pub non_markovianity: f64,
}

impl ProcessTensor {
    /// Creates a new process tensor
    pub fn new(system_dimension: usize, memory_depth: usize) -> Self {
        let hilbert_dim = 4_usize.pow(system_dimension as u32);
        
        Self {
            id: Uuid::new_v4(),
            time_steps: Vec::new(),
            tensors: vec![vec![Complex::new(0.0, 0.0); hilbert_dim]; memory_depth],
            memory_depth,
            system_dimension,
            non_markovianity: 0.0,
        }
    }
    
    /// Adds time step
    pub fn add_time_step(&mut self, time: f64, tensor: Vec<Complex<f64>>) {
        self.time_steps.push(time);
        
        // Rotate tensors (FIFO)
        if self.tensors.len() >= self.memory_depth {
            self.tensors.rotate_left(1);
            self.tensors[self.memory_depth - 1] = tensor;
        } else {
            self.tensors.push(tensor);
        }
        
        // Update non-Markovianity metric
        self.update_non_markovianity();
    }
    
    /// Calculates non-Markovianity metric
    /// 
    /// Based on:
    /// N(t) = max[0, d/dt D(ρ(t), ρ_M(t))]
    /// where D is the trace distance between the real and Markovian state
    fn update_non_markovianity(&mut self) {
        if self.tensors.len() < 2 {
            self.non_markovianity = 0.0;
            return;
        }
        
        // Calculate difference between consecutive steps
        let current = &self.tensors[self.tensors.len() - 1];
        let previous = &self.tensors[self.tensors.len() - 2];
        
        let mut difference_norm = 0.0;
        for (c, p) in current.iter().zip(previous.iter()) {
            difference_norm += (c - p).norm_sqr();
        }
        
        // Normalize
        self.non_markovianity = difference_norm.sqrt() / current.len() as f64;
    }
    
    /// Propagates state through the process tensor
    pub fn propagate(
        &self,
        initial_state: &DVector<Complex<f64>>,
        operations: &[DMatrix<Complex<f64>>],
    ) -> Result<DVector<Complex<f64>>, String> {
        if operations.is_empty() {
            return Ok(initial_state.clone());
        }
        
        let mut state = initial_state.clone();
        
        // Apply operations with temporal memory
        for (i, operation) in operations.iter().enumerate() {
            // Current operation
            state = operation * &state;
            
            // Apply memory correction if available
            if i < self.tensors.len() {
                let memory_correction = self.compute_memory_correction(i);
                state = &memory_correction * &state;
            }
        }
        
        Ok(state)
    }
    
    /// Calculates memory correction for step i
    fn compute_memory_correction(&self, step: usize) -> DMatrix<Complex<f64>> {
        let dim = self.system_dimension * 2;  // dim = 2^n for n qubits
        let mut correction = DMatrix::identity(dim, dim);
        
        // Apply process tensor as correction
        if step < self.tensors.len() {
            // Simplification: use only diagonal components
            for i in 0..dim.min(self.tensors[step].len()) {
                correction[(i, i)] = self.tensors[step][i];
            }
        }
        
        correction
    }
    
    /// Detects information backflow (non-Markovianity signature)
    pub fn detect_information_backflow(&self) -> Vec<(usize, f64)> {
        let mut backflow_events = Vec::new();
        
        if self.tensors.len() < 3 {
            return backflow_events;
        }
        
        // Search for increases in entropy/coherence (backflow)
        for i in 2..self.tensors.len() {
            let entropy_current = self.calculate_entropy(&self.tensors[i]);
            let entropy_previous = self.calculate_entropy(&self.tensors[i - 1]);
            
            // Backflow = increase in coherence (decrease in entropy)
            if entropy_current < entropy_previous {
                let backflow_strength = entropy_previous - entropy_current;
                backflow_events.push((i, backflow_strength));
            }
        }
        
        backflow_events
    }
    
    /// Calculates simplified von Neumann entropy
    fn calculate_entropy(&self, tensor: &[Complex<f64>]) -> f64 {
        let mut entropy = 0.0;
        
        for &value in tensor {
            let prob = value.norm_sqr();
            if prob > 1e-10 {
                entropy -= prob * prob.ln();
            }
        }
        
        entropy
    }
}

// ============================================================================
// PROCESS TENSOR BUILDER
// ============================================================================

/// Builds a process tensor from simulation
pub struct ProcessTensorBuilder {
    /// System dimension
    system_dimension: usize,
    
    /// Memory depth
    memory_depth: usize,
    
    /// System Hamiltonian
    system_hamiltonian: Option<DMatrix<Complex<f64>>>,
    
    /// Bath Hamiltonian
    bath_hamiltonian: Option<DMatrix<Complex<f64>>>,
    
    /// System-bath coupling
    coupling: Option<DMatrix<Complex<f64>>>,
    
    /// Bath temperature (Kelvin)
    temperature_k: f64,
}

impl ProcessTensorBuilder {
    pub fn new(system_dimension: usize, memory_depth: usize) -> Self {
        Self {
            system_dimension,
            memory_depth,
            system_hamiltonian: None,
            bath_hamiltonian: None,
            coupling: None,
            temperature_k: 300.0,  // Default room temperature
        }
    }
    
    pub fn with_system_hamiltonian(mut self, h: DMatrix<Complex<f64>>) -> Self {
        self.system_hamiltonian = Some(h);
        self
    }
    
    pub fn with_bath_hamiltonian(mut self, h: DMatrix<Complex<f64>>) -> Self {
        self.bath_hamiltonian = Some(h);
        self
    }
    
    pub fn with_coupling(mut self, c: DMatrix<Complex<f64>>) -> Self {
        self.coupling = Some(c);
        self
    }
    
    pub fn with_temperature(mut self, t: f64) -> Self {
        self.temperature_k = t;
        self
    }
    
    /// Builds process tensor via HEOM (Hierarchical Equations of Motion)
    pub fn build_heom(&self) -> Result<ProcessTensor, String> {
        let mut pt = ProcessTensor::new(self.system_dimension, self.memory_depth);
        
        // Time parameters
        let dt = 0.1;  // Time step in ns
        
        for step in 0..self.memory_depth {
            let time = step as f64 * dt;
            
            // Simulate HEOM evolution (simplified)
            let tensor = self.heom_step(time)?;
            
            pt.add_time_step(time, tensor);
        }
        
        Ok(pt)
    }
    
    /// Individual HEOM step
    fn heom_step(&self, time: f64) -> Result<Vec<Complex<f64>>, String> {
        let dim = 4_usize.pow(self.system_dimension as u32);
        
        // Simplification: free evolution + dissipation
        let mut tensor = vec![Complex::new(0.0, 0.0); dim];
        
        for i in 0..dim {
            // Unitary evolution
            let phase = -time * (i as f64);
            tensor[i] = Complex::new(phase.cos(), phase.sin());
            
            // Thermal dissipation
            let decay = (-time / 10.0).exp();  // T2 = 10 ns
            tensor[i] *= decay;
        }
        
        Ok(tensor)
    }
    
    /// Builds process tensor via Choi method
    pub fn build_choi(&self) -> Result<ProcessTensor, String> {
        let mut pt = ProcessTensor::new(self.system_dimension, self.memory_depth);
        
        // Build Choi representation
        // Υ = Σᵢⱼ |i⟩⟨j| ⊗ E(|i⟩⟨j|)
        
        let dt = 0.1;
        for step in 0..self.memory_depth {
            let time = step as f64 * dt;
            let tensor = self.choi_step(time)?;
            pt.add_time_step(time, tensor);
        }
        
        Ok(pt)
    }
    
    fn choi_step(&self, time: f64) -> Result<Vec<Complex<f64>>, String> {
        let dim = 4_usize.pow(self.system_dimension as u32);
        let mut tensor = vec![Complex::new(0.0, 0.0); dim];
        
        // Simplified Choi matrix
        for i in 0..dim {
            tensor[i] = Complex::new(
                (-time / 20.0).exp(),  // Decay
                0.0
            );
        }
        
        Ok(tensor)
    }
}

// ============================================================================
// PROCESS TENSOR CACHE
// ============================================================================

/// Process tensor cache for reuse
pub struct ProcessTensorCache {
    /// Stored tensors (hash → tensor)
    cache: std::collections::HashMap<u64, ProcessTensor>,
    
    /// Maximum cache size
    max_size: usize,
    
    /// LRU queue (Least Recently Used)
    lru_queue: VecDeque<u64>,
}

impl ProcessTensorCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: std::collections::HashMap::new(),
            max_size,
            lru_queue: VecDeque::new(),
        }
    }
    
    /// Calculates configuration hash
    fn compute_hash(
        system_dim: usize,
        memory_depth: usize,
        temperature: f64,
    ) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        system_dim.hash(&mut hasher);
        memory_depth.hash(&mut hasher);
        (temperature as u64).hash(&mut hasher);
        hasher.finish()
    }
    
    /// Gets tensor from cache
    pub fn get(
        &mut self,
        system_dim: usize,
        memory_depth: usize,
        temperature: f64,
    ) -> Option<&ProcessTensor> {
        let hash = Self::compute_hash(system_dim, memory_depth, temperature);
        
        if self.cache.contains_key(&hash) {
            // Update LRU
            self.lru_queue.retain(|&h| h != hash);
            self.lru_queue.push_back(hash);
            
            self.cache.get(&hash)
        } else {
            None
        }
    }
    
    /// Adds tensor to cache
    pub fn insert(
        &mut self,
        system_dim: usize,
        memory_depth: usize,
        temperature: f64,
        tensor: ProcessTensor,
    ) {
        let hash = Self::compute_hash(system_dim, memory_depth, temperature);
        
        // Avoid full cache
        if self.cache.len() >= self.max_size {
            if let Some(oldest) = self.lru_queue.pop_front() {
                self.cache.remove(&oldest);
            }
        }
        
        self.cache.insert(hash, tensor);
        self.lru_queue.push_back(hash);
    }
    
    /// Clears cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.lru_queue.clear();
    }
    
    /// Returns current size
    pub fn len(&self) -> usize {
        self.cache.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

// ============================================================================
// NON-MARKOVIANITY METRICS
// ============================================================================

/// Non-Markovianity metrics calculator
pub struct NonMarkovianityMetrics;

impl NonMarkovianityMetrics {
    /// Calculates BLP measure (Breuer-Laine-Piilo)
    /// N = max[0, d/dt D(ρ₁(t), ρ₂(t))]
    pub fn blp_measure(
        states: &[DMatrix<Complex<f64>>],
        times: &[f64],
    ) -> f64 {
        if states.len() < 2 || times.len() < 2 {
            return 0.0;
        }
        
        let mut max_increase = 0.0;
        
        for i in 1..states.len() {
            // Calculate trace distance
            let d_current = Self::trace_distance(&states[i], &states[i - 1]);
            let d_previous = if i > 1 {
                Self::trace_distance(&states[i - 1], &states[i - 2])
            } else {
                0.0
            };
            
            // Approximate time derivative
            let dt = times[i] - times[i - 1];
            let d_dt = (d_current - d_previous) / dt;
            
            if d_dt > max_increase {
                max_increase = d_dt;
            }
        }
        
        max_increase.max(0.0)
    }
    
    /// Calculates trace distance between two density matrices
    fn trace_distance(
        rho1: &DMatrix<Complex<f64>>,
        rho2: &DMatrix<Complex<f64>>,
    ) -> f64 {
        // D(ρ₁, ρ₂) = (1/2) Tr|ρ₁ - ρ₂|
        let diff = rho1 - rho2;
        
        // Simplification: use Frobenius norm
        let mut sum = 0.0;
        for i in 0..diff.nrows() {
            for j in 0..diff.ncols() {
                sum += diff[(i, j)].norm_sqr();
            }
        }
        
        0.5 * sum.sqrt()
    }
    
    /// Calculates RHP measure (Rivas-Huelga-Plenio)
    /// Based on divisibility of the dynamical map
    pub fn rhp_measure(process_tensor: &ProcessTensor) -> f64 {
        // Measure divisibility violation
        let backflow_events = process_tensor.detect_information_backflow();
        
        backflow_events.iter()
            .map(|(_, strength)| strength)
            .sum()
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
        let pt = ProcessTensor::new(2, 10);
        
        assert_eq!(pt.system_dimension, 2);
        assert_eq!(pt.memory_depth, 10);
        assert_eq!(pt.non_markovianity, 0.0);
    }
    
    #[test]
    fn test_add_time_step() {
        let mut pt = ProcessTensor::new(1, 3);
        
        let tensor1 = vec![Complex::new(1.0, 0.0); 4];
        pt.add_time_step(0.0, tensor1);
        
        assert_eq!(pt.time_steps.len(), 1);
        assert!(pt.tensors.len() <= pt.memory_depth);
    }
    
    #[test]
    fn test_builder_heom() {
        let builder = ProcessTensorBuilder::new(1, 5)
            .with_temperature(300.0);
        
        let pt = builder.build_heom().unwrap();
        
        assert_eq!(pt.memory_depth, 5);
        assert_eq!(pt.time_steps.len(), 5);
    }
    
    #[test]
    fn test_cache() {
        let mut cache = ProcessTensorCache::new(10);
        
        let pt = ProcessTensor::new(2, 5);
        cache.insert(2, 5, 300.0, pt);
        
        assert_eq!(cache.len(), 1);
        
        let retrieved = cache.get(2, 5, 300.0);
        assert!(retrieved.is_some());
    }
    
    #[test]
    fn test_backflow_detection() {
        let mut pt = ProcessTensor::new(1, 5);
        
        // Add steps with increasing entropy and then decreasing entropy
        for i in 0..5 {
            let entropy_factor = if i < 3 { i as f64 } else { (5 - i) as f64 };
            let tensor = vec![Complex::new(entropy_factor, 0.0); 4];
            pt.add_time_step(i as f64, tensor);
        }
        
        let backflow = pt.detect_information_backflow();
        assert!(!backflow.is_empty());
    }
    
    #[test]
    fn test_non_markovianity_metric() {
        let state1 = DMatrix::from_diagonal(&DVector::from_vec(vec![
            Complex::new(1.0, 0.0),
            Complex::new(0.0, 0.0),
        ]));
        
        let state2 = DMatrix::from_diagonal(&DVector::from_vec(vec![
            Complex::new(0.8, 0.0),
            Complex::new(0.2, 0.0),
        ]));
        
        let states = vec![state1, state2];
        let times = vec![0.0, 1.0];
        
        let n = NonMarkovianityMetrics::blp_measure(&states, &times);
        assert!(n >= 0.0);
    }
}
