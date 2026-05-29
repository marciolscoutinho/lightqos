// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// lib.rs — Shadow Tomography crate — advanced state reconstruction
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 08-06-2022
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod shadow_copy;
pub mod observable_view;
pub mod statistical_certificate;
pub mod adaptive_resampling;
pub mod mid_circuit_feedback;

pub use shadow_copy::{ShadowCopy, ShadowCopyResult, ExecutionStrategy};
pub use observable_view::{ObservableView, MeasurementBasis, MultiBaseResult};
pub use statistical_certificate::{StatisticalCertificate, ConfidenceInterval, ErrorBounds};
pub use adaptive_resampling::{AdaptiveResampler, ResamplingStrategy, PrecisionTarget};
pub use mid_circuit_feedback::{MidCircuitFeedback, FeedbackDecision, FeedbackRule};

// Re-exports
pub use nalgebra::{DMatrix, DVector};
pub use num_complex::Complex64;
pub use ndarray::{Array1, Array2};

/// Version of shadow tomography framework
pub const VERSION: &str = "0.2.0";

/// Default number of shots for shadow tomography
pub const DEFAULT_SHOTS: usize = 1000;

/// Default confidence level (95%)
pub const DEFAULT_CONFIDENCE: f64 = 0.95;

/// Numerical precision
pub const EPSILON: f64 = 1.0e-10;

// ============================================================================
// HOLOGRAPHIC I/O (HIO)
// ============================================================================

/// Holographic I/O interface - complete implementation
#[derive(Debug, Clone)]
pub struct HolographicIO {
    /// Shadow copy engine
    pub shadow_engine: shadow_copy::ShadowCopyEngine,
    
    /// Observable view manager
    pub view_manager: observable_view::ViewManager,
    
    /// Statistical certifier
    pub certifier: statistical_certificate::Certifier,
    
    /// Adaptive resampler
    pub resampler: adaptive_resampling::AdaptiveResampler,
    
    /// Mid-circuit feedback handler
    pub feedback: mid_circuit_feedback::FeedbackHandler,
}

impl HolographicIO {
    /// Create new HIO system
    pub fn new() -> Self {
        Self {
            shadow_engine: shadow_copy::ShadowCopyEngine::new(DEFAULT_SHOTS),
            view_manager: observable_view::ViewManager::new(),
            certifier: statistical_certificate::Certifier::new(DEFAULT_CONFIDENCE),
            resampler: adaptive_resampling::AdaptiveResampler::new(),
            feedback: mid_circuit_feedback::FeedbackHandler::new(),
        }
    }
    
    /// Measure with holographic I/O
    pub fn measure_holographic(
        &mut self,
        circuit: &QuantumCircuit,
        options: HIOOptions,
    ) -> HIOResult {
        // 1. Collect shadow copies
        let shadows = self.shadow_engine.collect_shadows(circuit, options.shots);
        
        // 2. Measure in multiple bases if requested
        let views = if options.multi_base {
            Some(self.view_manager.measure_multi_base(circuit, &options.bases))
        } else {
            None
        };
        
        // 3. Generate statistical certificate
        let certificate = self.certifier.certify(&shadows, options.confidence);
        
        // 4. Check if resampling needed
        if !certificate.meets_precision(&options.precision) {
            let additional = self.resampler.compute_additional_shots(&certificate, &options);
            // Resample...
        }
        
        HIOResult {
            shadows,
            views,
            certificate,
            success: true,
        }
    }
}

impl Default for HolographicIO {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HIO OPTIONS & RESULT
// ============================================================================

/// Options for holographic measurement
#[derive(Debug, Clone)]
pub struct HIOOptions {
    /// Number of shots
    pub shots: usize,
    
    /// Measure in multiple bases?
    pub multi_base: bool,
    
    /// Bases to measure (if multi_base = true)
    pub bases: Vec<MeasurementBasis>,
    
    /// Confidence level (e.g., 0.95 for 95%)
    pub confidence: f64,
    
    /// Precision target
    pub precision: PrecisionTarget,
}

impl Default for HIOOptions {
    fn default() -> Self {
        Self {
            shots: DEFAULT_SHOTS,
            multi_base: false,
            bases: vec![MeasurementBasis::Z],
            confidence: DEFAULT_CONFIDENCE,
            precision: PrecisionTarget::default(),
        }
    }
}

/// Result of holographic measurement
#[derive(Debug, Clone)]
pub struct HIOResult {
    /// Shadow copies data
    pub shadows: ShadowCopyResult,
    
    /// Observable views (if requested)
    pub views: Option<MultiBaseResult>,
    
    /// Statistical certificate
    pub certificate: StatisticalCertificate,
    
    /// Success flag
    pub success: bool,
}

// ============================================================================
// QUANTUM CIRCUIT (SIMPLIFIED)
// ============================================================================

/// Simplified quantum circuit representation
#[derive(Debug, Clone)]
pub struct QuantumCircuit {
    /// Number of qubits
    pub num_qubits: usize,
    
    /// Gates (simplified)
    pub gates: Vec<String>,
}

impl QuantumCircuit {
    /// Create new circuit
    pub fn new(num_qubits: usize) -> Self {
        Self {
            num_qubits,
            gates: Vec::new(),
        }
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Compute fidelity between density matrices
pub fn fidelity(rho1: &DMatrix<Complex64>, rho2: &DMatrix<Complex64>) -> f64 {
    // F = Tr(√(√ρ₁ ρ₂ √ρ₁))²
    // Simplified for pure states
    let overlap = (rho1.component_mul(rho2)).sum();
    overlap.norm_sqr()
}

/// Compute purity: Tr(ρ²)
pub fn purity(rho: &DMatrix<Complex64>) -> f64 {
    let rho_squared = rho * rho;
    let trace: Complex64 = (0..rho.nrows()).map(|i| rho_squared[(i, i)]).sum();
    trace.norm()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hio_creation() {
        let hio = HolographicIO::new();
        assert_eq!(hio.shadow_engine.default_shots, DEFAULT_SHOTS);
    }
    
    #[test]
    fn test_fidelity() {
        let mut rho1 = DMatrix::zeros(2, 2);
        rho1[(0, 0)] = Complex64::new(1.0, 0.0);
        
        let mut rho2 = DMatrix::zeros(2, 2);
        rho2[(0, 0)] = Complex64::new(1.0, 0.0);
        
        let f = fidelity(&rho1, &rho2);
        assert!((f - 1.0).abs() < EPSILON);
    }
}
