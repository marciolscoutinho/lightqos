// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// adaptive_resampling.rs — Adaptive Resampling — quality-driven shadow refinement
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 22-11-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::statistical_certificate::StatisticalCertificate;
use crate::HIOOptions;
use uuid::Uuid;

// ============================================================================
// ADAPTIVE RESAMPLER
// ============================================================================

/// Adaptive resampling engine
#[derive(Debug, Clone)]
pub struct AdaptiveResampler {
    /// Resampling strategy
    pub strategy: ResamplingStrategy,
    
    /// Maximum iterations
    pub max_iterations: usize,
    
    /// Current iteration
    pub current_iteration: usize,
    
    /// History of resamplings
    pub history: Vec<ResamplingRecord>,
}

impl AdaptiveResampler {
    /// Create new adaptive resampler
    pub fn new() -> Self {
        Self {
            strategy: ResamplingStrategy::Conservative,
            max_iterations: 10,
            current_iteration: 0,
            history: Vec::new(),
        }
    }
    
    /// Compute additional shots needed
    pub fn compute_additional_shots(
        &mut self,
        certificate: &StatisticalCertificate,
        options: &HIOOptions,
    ) -> AdditionalShots {
        self.current_iteration += 1;
        
        // Check if max iterations reached
        if self.current_iteration >= self.max_iterations {
            return AdditionalShots {
                shots_needed: 0,
                reason: ResamplingReason::MaxIterationsReached,
                confidence: 0.0,
            };
        }
        
        // Estimate shots needed based on current error
        let shots_needed = self.estimate_shots_needed(
            certificate,
            &options.precision,
            options.confidence,
        );
        
        // Apply strategy adjustment
        let adjusted_shots = self.apply_strategy(shots_needed);
        
        // Record this resampling
        let record = ResamplingRecord {
            id: Uuid::new_v4(),
            iteration: self.current_iteration,
            current_shots: certificate.shots,
            additional_shots: adjusted_shots,
            current_error: certificate.error_bounds.standard_error,
            target_error: options.precision.max_std_error,
        };
        self.history.push(record);
        
        AdditionalShots {
            shots_needed: adjusted_shots,
            reason: ResamplingReason::PrecisionNotMet,
            confidence: self.estimate_confidence(adjusted_shots, certificate),
        }
    }
    
    /// Estimate shots needed to reach target precision
    fn estimate_shots_needed(
        &self,
        certificate: &StatisticalCertificate,
        target: &PrecisionTarget,
        confidence: f64,
    ) -> usize {
        let current_error = certificate.error_bounds.standard_error;
        let target_error = target.max_std_error;
        
        if current_error <= target_error {
            return 0;
        }
        
        // Error scales as 1/√n, so:
        // target_error = current_error * √(n_current / n_needed)
        // n_needed = n_current * (current_error / target_error)²
        
        let n_current = certificate.shots as f64;
        let ratio = current_error / target_error;
        let n_needed = (n_current * ratio.powi(2)).ceil() as usize;
        
        // Additional shots = n_needed - n_current
        n_needed.saturating_sub(certificate.shots)
    }
    
    /// Apply resampling strategy adjustment
    fn apply_strategy(&self, base_shots: usize) -> usize {
        match self.strategy {
            ResamplingStrategy::Conservative => {
                // Add 20% buffer
                (base_shots as f64 * 1.2).ceil() as usize
            }
            ResamplingStrategy::Moderate => {
                // Add 10% buffer
                (base_shots as f64 * 1.1).ceil() as usize
            }
            ResamplingStrategy::Aggressive => {
                // No buffer
                base_shots
            }
            ResamplingStrategy::Adaptive { factor } => {
                // Custom factor
                (base_shots as f64 * factor).ceil() as usize
            }
        }
    }
    
    /// Estimate confidence with additional shots
    fn estimate_confidence(&self, additional_shots: usize, certificate: &StatisticalCertificate) -> f64 {
        let total_shots = certificate.shots + additional_shots;
        let improvement_factor = (total_shots as f64 / certificate.shots as f64).sqrt();
        
        // Estimate new error
        let new_error = certificate.error_bounds.standard_error / improvement_factor;
        
        // Map error to confidence (simplified)
        if new_error < 0.01 {
            0.99
        } else if new_error < 0.05 {
            0.95
        } else {
            0.90
        }
    }
    
    /// Reset resampler
    pub fn reset(&mut self) {
        self.current_iteration = 0;
        self.history.clear();
    }
    
    /// Get total shots across all iterations
    pub fn total_shots(&self) -> usize {
        self.history.iter()
            .map(|r| r.current_shots + r.additional_shots)
            .sum()
    }
}

impl Default for AdaptiveResampler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// RESAMPLING STRATEGY
// ============================================================================

/// Strategy for adaptive resampling
#[derive(Debug, Clone, Copy)]
pub enum ResamplingStrategy {
    /// Conservative: add 20% buffer
    Conservative,
    
    /// Moderate: add 10% buffer
    Moderate,
    
    /// Aggressive: no buffer
    Aggressive,
    
    /// Adaptive with custom factor
    Adaptive { factor: f64 },
}

// ============================================================================
// PRECISION TARGET
// ============================================================================

/// Target precision for measurements
#[derive(Debug, Clone, Copy)]
pub struct PrecisionTarget {
    /// Maximum standard error allowed
    pub max_std_error: f64,
    
    /// Maximum relative error allowed (percentage)
    pub max_relative_error: f64,
    
    /// Minimum confidence level required
    pub min_confidence: f64,
}

impl Default for PrecisionTarget {
    fn default() -> Self {
        Self {
            max_std_error: 0.01,      // 1% absolute error
            max_relative_error: 0.05,  // 5% relative error
            min_confidence: 0.95,      // 95% confidence
        }
    }
}

impl PrecisionTarget {
    /// Create strict precision target
    pub fn strict() -> Self {
        Self {
            max_std_error: 0.001,
            max_relative_error: 0.01,
            min_confidence: 0.99,
        }
    }
    
    /// Create relaxed precision target
    pub fn relaxed() -> Self {
        Self {
            max_std_error: 0.05,
            max_relative_error: 0.10,
            min_confidence: 0.90,
        }
    }
}

// ============================================================================
// ADDITIONAL SHOTS
// ============================================================================

/// Additional shots recommendation
#[derive(Debug, Clone, Copy)]
pub struct AdditionalShots {
    /// Number of additional shots needed
    pub shots_needed: usize,
    
    /// Reason for resampling
    pub reason: ResamplingReason,
    
    /// Expected confidence after resampling
    pub confidence: f64,
}

// ============================================================================
// RESAMPLING REASON
// ============================================================================

/// Reason for requesting resampling
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResamplingReason {
    /// Precision target not met
    PrecisionNotMet,
    
    /// Confidence level too low
    LowConfidence,
    
    /// Maximum iterations reached
    MaxIterationsReached,
    
    /// User requested
    UserRequested,
}

// ============================================================================
// RESAMPLING RECORD
// ============================================================================

/// Record of a resampling iteration
#[derive(Debug, Clone)]
pub struct ResamplingRecord {
    /// Unique ID
    pub id: Uuid,
    
    /// Iteration number
    pub iteration: usize,
    
    /// Shots before resampling
    pub current_shots: usize,
    
    /// Additional shots added
    pub additional_shots: usize,
    
    /// Error before resampling
    pub current_error: f64,
    
    /// Target error
    pub target_error: f64,
}

impl ResamplingRecord {
    /// Improvement factor
    pub fn improvement_factor(&self) -> f64 {
        let total_shots = self.current_shots + self.additional_shots;
        (total_shots as f64 / self.current_shots as f64).sqrt()
    }
    
    /// Expected new error
    pub fn expected_error(&self) -> f64 {
        self.current_error / self.improvement_factor()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_resampler_creation() {
        let resampler = AdaptiveResampler::new();
        assert_eq!(resampler.max_iterations, 10);
        assert_eq!(resampler.current_iteration, 0);
    }
    
    #[test]
    fn test_precision_targets() {
        let default_target = PrecisionTarget::default();
        let strict_target = PrecisionTarget::strict();
        let relaxed_target = PrecisionTarget::relaxed();
        
        assert!(strict_target.max_std_error < default_target.max_std_error);
        assert!(relaxed_target.max_std_error > default_target.max_std_error);
    }
    
    #[test]
    fn test_strategy_adjustment() {
        let resampler = AdaptiveResampler::new();
        
        let base = 1000;
        let conservative = (base as f64 * 1.2) as usize;
        
        assert_eq!(resampler.apply_strategy(base), conservative);
    }
    
    #[test]
    fn test_resampling_record() {
        let record = ResamplingRecord {
            id: Uuid::new_v4(),
            iteration: 1,
            current_shots: 1000,
            additional_shots: 1000,
            current_error: 0.02,
            target_error: 0.01,
        };
        
        assert!((record.improvement_factor() - 1.414).abs() < 0.01);
        assert!(record.expected_error() < record.current_error);
    }
}
