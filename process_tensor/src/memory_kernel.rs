// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// memory_kernel.rs — Memory Kernel — non-Markovian bath correlation functions
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 21-06-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::DMatrix;
use num_complex::Complex64;
use std::collections::HashMap;

// ============================================================================
// MEMORY KERNEL
// ============================================================================

/// Memory kernel for non-Markovian dynamics
#[derive(Debug, Clone)]
pub struct MemoryKernel {
    /// Kernel type
    pub kernel_type: KernelType,
    
    /// Correlation function
    pub correlation: CorrelationFunction,
    
    /// Memory time scale (τ_memory)
    pub memory_timescale: f64,
    
    /// Strength of memory effects
    pub strength: f64,
    
    /// System dimension
    pub system_dim: usize,
}

impl MemoryKernel {
    /// Create new memory kernel
    pub fn new(
        kernel_type: KernelType,
        memory_timescale: f64,
        strength: f64,
        system_dim: usize,
    ) -> Self {
        let correlation = match kernel_type {
            KernelType::Exponential => {
                CorrelationFunction::exponential(memory_timescale, strength)
            }
            KernelType::Lorentzian => {
                CorrelationFunction::lorentzian(memory_timescale, strength)
            }
            KernelType::Ohmic => {
                CorrelationFunction::ohmic(memory_timescale, strength)
            }
        };
        
        Self {
            kernel_type,
            correlation,
            memory_timescale,
            strength,
            system_dim,
        }
    }
    
    /// Apply memory effect to current state
    pub fn apply_memory_effect(
        &self,
        current_state: &DMatrix<Complex64>,
        current_time: f64,
        past_states: &HashMap<usize, DMatrix<Complex64>>,
    ) -> DMatrix<Complex64> {
        let mut state = current_state.clone();
        
        // Memory integral: ∫₀ᵗ K(t-s) ρ(s) ds
        for (past_time_idx, past_state) in past_states {
            let time_diff = current_time - (*past_time_idx as f64);
            
            if time_diff > 0.0 {
                let kernel_value = self.correlation.evaluate(time_diff);
                
                // Apply memory contribution
                let memory_contrib = past_state * kernel_value;
                state += memory_contrib * self.strength;
            }
        }
        
        // Renormalize to maintain trace
        self.normalize_trace(&mut state);
        
        state
    }
    
    /// Normalize trace of state
    fn normalize_trace(&self, state: &mut DMatrix<Complex64>) {
        let trace: Complex64 = (0..state.nrows()).map(|i| state[(i, i)]).sum();
        let norm = trace.norm();
        
        if norm > 1.0e-10 {
            *state /= norm;
        }
    }
    
    /// Memory strength measure
    pub fn memory_strength(&self) -> f64 {
        self.strength
    }
    
    /// Check if process is non-Markovian
    pub fn is_non_markovian(&self) -> bool {
        self.strength > 1.0e-10
    }
    
    /// Compute non-Markovianity measure (BLP)
    pub fn blp_measure(&self, time_window: f64) -> f64 {
        // Breuer-Laine-Piilo measure
        // N = max(0, ∫ σ'(t) dt) where σ'(t) > 0 indicates backflow
        
        // Simplified: proportional to memory strength and timescale
        self.strength * self.memory_timescale.min(time_window)
    }
}

// ============================================================================
// KERNEL TYPE
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KernelType {
    /// Exponential decay: K(t) = e^(-t/τ)
    Exponential,
    
    /// Lorentzian: K(t) = 1/(1 + (t/τ)²)
    Lorentzian,
    
    /// Ohmic spectral density
    Ohmic,
}

// ============================================================================
// CORRELATION FUNCTION
// ============================================================================

/// Bath correlation function
#[derive(Debug, Clone)]
pub struct CorrelationFunction {
    /// Function type
    pub function_type: CorrelationFunctionType,
    
    /// Time scale parameter
    pub timescale: f64,
    
    /// Amplitude
    pub amplitude: f64,
    
    /// Temperature (for thermal baths)
    pub temperature: Option<f64>,
}

impl CorrelationFunction {
    /// Exponential correlation
    pub fn exponential(timescale: f64, amplitude: f64) -> Self {
        Self {
            function_type: CorrelationFunctionType::Exponential,
            timescale,
            amplitude,
            temperature: None,
        }
    }
    
    /// Lorentzian correlation
    pub fn lorentzian(timescale: f64, amplitude: f64) -> Self {
        Self {
            function_type: CorrelationFunctionType::Lorentzian,
            timescale,
            amplitude,
            temperature: None,
        }
    }
    
    /// Ohmic spectral density
    pub fn ohmic(cutoff_freq: f64, coupling: f64) -> Self {
        Self {
            function_type: CorrelationFunctionType::Ohmic,
            timescale: cutoff_freq,
            amplitude: coupling,
            temperature: None,
        }
    }
    
    /// Evaluate correlation at time t
    pub fn evaluate(&self, time: f64) -> f64 {
        match self.function_type {
            CorrelationFunctionType::Exponential => {
                // C(t) = A e^(-t/τ)
                self.amplitude * (-time / self.timescale).exp()
            }
            CorrelationFunctionType::Lorentzian => {
                // C(t) = A / (1 + (t/τ)²)
                self.amplitude / (1.0 + (time / self.timescale).powi(2))
            }
            CorrelationFunctionType::Ohmic => {
                // J(ω) = α ω e^(-ω/ωc)
                // Simplified time-domain version
                let omega = time;
                self.amplitude * omega * (-omega / self.timescale).exp()
            }
            CorrelationFunctionType::Gaussian => {
                // C(t) = A e^(-(t/τ)²)
                self.amplitude * (-(time / self.timescale).powi(2)).exp()
            }
        }
    }
    
    /// Power spectrum (Fourier transform of correlation)
    pub fn power_spectrum(&self, frequency: f64) -> f64 {
        match self.function_type {
            CorrelationFunctionType::Exponential => {
                // S(ω) = 2Aτ / (1 + (ωτ)²)
                2.0 * self.amplitude * self.timescale 
                    / (1.0 + (frequency * self.timescale).powi(2))
            }
            CorrelationFunctionType::Lorentzian => {
                // Approximation
                self.amplitude / (1.0 + frequency.powi(2))
            }
            CorrelationFunctionType::Ohmic => {
                // J(ω) = α ω e^(-ω/ωc)
                self.amplitude * frequency * (-frequency / self.timescale).exp()
            }
            CorrelationFunctionType::Gaussian => {
                // S(ω) = A√π τ e^(-(ωτ)²/4)
                let sqrt_pi = std::f64::consts::PI.sqrt();
                self.amplitude * sqrt_pi * self.timescale 
                    * (-(frequency * self.timescale).powi(2) / 4.0).exp()
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrelationFunctionType {
    Exponential,
    Lorentzian,
    Ohmic,
    Gaussian,
}

// ============================================================================
// NON-MARKOVIAN DYNAMICS
// ============================================================================

/// Non-Markovian dynamics handler
#[derive(Debug, Clone)]
pub struct NonMarkovianDynamics {
    /// Memory kernel
    pub kernel: MemoryKernel,
    
    /// History of states
    pub state_history: HashMap<usize, DMatrix<Complex64>>,
    
    /// Time step
    pub dt: f64,
    
    /// Current time index
    pub current_time: usize,
}

impl NonMarkovianDynamics {
    /// Create new non-Markovian dynamics
    pub fn new(kernel: MemoryKernel, dt: f64) -> Self {
        Self {
            kernel,
            state_history: HashMap::new(),
            dt,
            current_time: 0,
        }
    }
    
    /// Evolve state one step
    pub fn evolve_step(
        &mut self,
        current_state: DMatrix<Complex64>,
    ) -> DMatrix<Complex64> {
        // Store current state in history
        self.state_history.insert(self.current_time, current_state.clone());
        
        // Apply memory effects
        let time = (self.current_time as f64) * self.dt;
        let evolved = self.kernel.apply_memory_effect(
            &current_state,
            time,
            &self.state_history,
        );
        
        self.current_time += 1;
        
        evolved
    }
    
    /// Reset dynamics
    pub fn reset(&mut self) {
        self.state_history.clear();
        self.current_time = 0;
    }
    
    /// Get state at past time
    pub fn get_past_state(&self, time_index: usize) -> Option<&DMatrix<Complex64>> {
        self.state_history.get(&time_index)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memory_kernel_creation() {
        let kernel = MemoryKernel::new(
            KernelType::Exponential,
            1.0,  // timescale
            0.5,  // strength
            2,    // dimension
        );
        
        assert!(kernel.is_non_markovian());
        assert_eq!(kernel.memory_timescale, 1.0);
    }
    
    #[test]
    fn test_correlation_exponential() {
        let corr = CorrelationFunction::exponential(1.0, 1.0);
        
        assert!((corr.evaluate(0.0) - 1.0).abs() < 1e-10);
        assert!(corr.evaluate(1.0) < 1.0);
        assert!(corr.evaluate(1.0) > 0.0);
    }
    
    #[test]
    fn test_correlation_lorentzian() {
        let corr = CorrelationFunction::lorentzian(1.0, 1.0);
        
        assert!((corr.evaluate(0.0) - 1.0).abs() < 1e-10);
        assert!((corr.evaluate(1.0) - 0.5).abs() < 1e-10);
    }
    
    #[test]
    fn test_non_markovian_dynamics() {
        let kernel = MemoryKernel::new(
            KernelType::Exponential,
            1.0,
            0.1,
            2,
        );
        
        let mut dynamics = NonMarkovianDynamics::new(kernel, 0.1);
        
        let initial_state = DMatrix::identity(2, 2);
        let evolved = dynamics.evolve_step(initial_state);
        
        assert_eq!(dynamics.current_time, 1);
        assert_eq!(evolved.nrows(), 2);
    }
}
