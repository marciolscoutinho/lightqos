// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// py_emf.rs — PyO3 Python bindings for EMF (Entangled Memory Fabric)
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 24-03-2026
// All rights reserved.
// ---------------------------------------------------------------------------

use pyo3::prelude::*;
use pyo3::types::PyDict;
use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// ENTANGLED PAIR (Python)
// ============================================================================

/// Entangled pair (Python version)
#[pyclass(name = "EntangledPair")]
#[derive(Clone)]
pub struct PyEntangledPair {
    #[pyo3(get)]
    pub id: String,
    
    #[pyo3(get)]
    pub qubit_a: usize,
    
    #[pyo3(get)]
    pub qubit_b: usize,
    
    #[pyo3(get)]
    pub fidelity: f64,
    
    #[pyo3(get)]
    pub age: u64,
}

#[pymethods]
impl PyEntangledPair {
    #[new]
    fn new(qubit_a: usize, qubit_b: usize, fidelity: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            qubit_a,
            qubit_b,
            fidelity,
            age: 0,
        }
    }
    
    /// Ages the pair (fidelity degradation)
    fn age_pair(&mut self) {
        self.age += 1;
        self.fidelity *= 0.99; // 1% degradation per timestep
    }
    
    /// Checks whether the pair is still usable
    fn is_usable(&self, threshold: f64) -> bool {
        self.fidelity >= threshold
    }
    
    /// String representation
    fn __repr__(&self) -> String {
        format!(
            "EntangledPair(id={}, qubits=({}, {}), F={:.3}, age={})",
            &self.id[..8],
            self.qubit_a,
            self.qubit_b,
            self.fidelity,
            self.age
        )
    }
}

// ============================================================================
// EMF MANAGER (Python)
// ============================================================================

/// EMF manager (Python version)
#[pyclass(name = "EMFManager")]
pub struct PyEMFManager {
    pairs: HashMap<String, PyEntangledPair>,
    max_pairs: usize,
    recycling_threshold: f64,
    
    // Statistics
    total_generated: u64,
    total_recycled: u64,
    total_consumed: u64,
}

#[pymethods]
impl PyEMFManager {
    #[new]
    fn new(max_pairs: usize, recycling_threshold: f64) -> Self {
        Self {
            pairs: HashMap::new(),
            max_pairs,
            recycling_threshold,
            total_generated: 0,
            total_recycled: 0,
            total_consumed: 0,
        }
    }
    
    /// Generates a new entangled pair
    fn generate_pair(&mut self, qubit_a: usize, qubit_b: usize, fidelity: f64) -> PyResult<String> {
        // Check capacity
        if self.pairs.len() >= self.max_pairs {
            return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(
                "EMF capacity exceeded"
            ));
        }
        
        // Create pair
        let pair = PyEntangledPair::new(qubit_a, qubit_b, fidelity);
        let id = pair.id.clone();
        
        self.pairs.insert(id.clone(), pair);
        self.total_generated += 1;
        
        Ok(id)
    }
    
    /// Gets a pair by ID
    fn get_pair(&self, id: String) -> PyResult<PyEntangledPair> {
        self.pairs.get(&id)
            .cloned()
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Pair {} not found", id)
            ))
    }
    
    /// Consumes pair (removes it)
    fn consume_pair(&mut self, id: String) -> PyResult<()> {
        self.pairs.remove(&id)
            .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyKeyError, _>(
                format!("Pair {} not found", id)
            ))?;
        
        self.total_consumed += 1;
        Ok(())
    }
    
    /// Ages all pairs
    fn age_all_pairs(&mut self) {
        for pair in self.pairs.values_mut() {
            pair.age_pair();
        }
    }
    
    /// Recycles pairs with low fidelity
    fn recycle(&mut self) -> usize {
        let threshold = self.recycling_threshold;
        
        // Identify pairs to recycle
        let to_recycle: Vec<String> = self.pairs.iter()
            .filter(|(_, p)| p.fidelity < threshold)
            .map(|(id, _)| id.clone())
            .collect();
        
        // Remove
        let count = to_recycle.len();
        for id in to_recycle {
            self.pairs.remove(&id);
        }
        
        self.total_recycled += count as u64;
        count
    }
    
    /// Returns the number of active pairs
    fn num_pairs(&self) -> usize {
        self.pairs.len()
    }
    
    /// Calculates mean fidelity
    fn avg_fidelity(&self) -> f64 {
        if self.pairs.is_empty() {
            return 0.0;
        }
        
        let sum: f64 = self.pairs.values()
            .map(|p| p.fidelity)
            .sum();
        
        sum / self.pairs.len() as f64
    }
    
    /// Returns statistics
    fn get_statistics(&self, py: Python) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        
        dict.set_item("total_generated", self.total_generated)?;
        dict.set_item("total_recycled", self.total_recycled)?;
        dict.set_item("total_consumed", self.total_consumed)?;
        dict.set_item("active_pairs", self.pairs.len())?;
        dict.set_item("avg_fidelity", self.avg_fidelity())?;
        dict.set_item("capacity", self.max_pairs)?;
        dict.set_item("utilization", self.pairs.len() as f64 / self.max_pairs as f64)?;
        
        Ok(dict.into())
    }
    
    /// Lists all IDs
    fn list_pair_ids(&self) -> Vec<String> {
        self.pairs.keys().cloned().collect()
    }
    
    /// String representation
    fn __repr__(&self) -> String {
        format!(
            "EMFManager(pairs={}/{}, F_avg={:.3})",
            self.pairs.len(),
            self.max_pairs,
            self.avg_fidelity()
        )
    }
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Calculates fidelity after N timesteps
#[pyfunction]
pub fn calculate_fidelity_decay(initial_fidelity: f64, timesteps: u64, decay_rate: f64) -> f64 {
    initial_fidelity * (1.0 - decay_rate).powi(timesteps as i32)
}

/// Calculates pressure (load/capacity)
#[pyfunction]
pub fn calculate_pressure(num_pairs: usize, capacity: usize) -> f64 {
    num_pairs as f64 / capacity as f64
}
