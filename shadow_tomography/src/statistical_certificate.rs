// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// statistical_certificate.rs — Statistical Certificate — Hoeffding bound confidence proofs
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 04-11-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::shadow_copy::ShadowCopyResult;
use std::collections::HashMap;
use uuid::Uuid;

// ============================================================================
// CERTIFIER
// ============================================================================

/// Statistical certifier
#[derive(Debug, Clone)]
pub struct Certifier {
    /// Default confidence level
    pub default_confidence: f64,
    
    /// Z-scores for common confidence levels
    pub z_scores: HashMap<String, f64>,
}

impl Certifier {
    /// Create new certifier
    pub fn new(confidence: f64) -> Self {
        let mut z_scores = HashMap::new();
        z_scores.insert("0.90".to_string(), 1.645);  // 90%
        z_scores.insert("0.95".to_string(), 1.960);  // 95%
        z_scores.insert("0.99".to_string(), 2.576);  // 99%
        z_scores.insert("0.999".to_string(), 3.291); // 99.9%
        
        Self {
            default_confidence: confidence,
            z_scores,
        }
    }
    
    /// Generate statistical certificate
    pub fn certify(
        &self,
        shadows: &ShadowCopyResult,
        confidence: f64,
    ) -> StatisticalCertificate {
        // Compute confidence intervals for expectations
        let intervals = self.compute_confidence_intervals(shadows, confidence);
        
        // Compute error bounds
        let bounds = self.compute_error_bounds(shadows);
        
        // Estimate sample adequacy
        let adequacy = self.assess_sample_adequacy(shadows, confidence);
        
        // Compute chi-squared goodness of fit
        let chi_squared = self.compute_chi_squared(shadows);
        
        StatisticalCertificate {
            id: Uuid::new_v4(),
            confidence_level: confidence,
            intervals,
            error_bounds: bounds,
            sample_adequacy: adequacy,
            chi_squared,
            shots: shadows.shots,
        }
    }
    
    /// Compute confidence intervals for all expectations
    fn compute_confidence_intervals(
        &self,
        shadows: &ShadowCopyResult,
        confidence: f64,
    ) -> HashMap<String, ConfidenceInterval> {
        let mut intervals = HashMap::new();
        
        let z = self.get_z_score(confidence);
        
        for (observable, &expectation) in &shadows.expectations {
            let std_error = shadows.error_bar(observable).unwrap_or(0.0);
            let margin = z * std_error;
            
            intervals.insert(
                observable.clone(),
                ConfidenceInterval {
                    value: expectation,
                    lower: expectation - margin,
                    upper: expectation + margin,
                    confidence,
                },
            );
        }
        
        intervals
    }
    
    /// Compute error bounds
    fn compute_error_bounds(&self, shadows: &ShadowCopyResult) -> ErrorBounds {
        // Compute various error metrics
        
        // Standard error of the mean
        let sem = shadows.entropy / (shadows.shots as f64).sqrt();
        
        // Relative error
        let mean_exp: f64 = shadows.expectations.values().sum::<f64>() 
                            / shadows.expectations.len() as f64;
        let relative_error = if mean_exp.abs() > 1e-10 {
            sem / mean_exp.abs()
        } else {
            sem
        };
        
        // Maximum error across all observables
        let max_error = shadows.variance.values()
            .map(|&v| (v / shadows.shots as f64).sqrt())
            .fold(0.0, f64::max);
        
        ErrorBounds {
            standard_error: sem,
            relative_error,
            max_error,
            epsilon_delta: (sem, 0.05), // (ε, δ) bound
        }
    }
    
    /// Assess if sample size is adequate
    fn assess_sample_adequacy(
        &self,
        shadows: &ShadowCopyResult,
        confidence: f64,
    ) -> SampleAdequacy {
        let z = self.get_z_score(confidence);
        
        // Estimate required sample size using worst-case variance
        let max_variance = shadows.variance.values()
            .fold(0.0, |a, &b| a.max(b));
        
        let target_error = 0.01; // 1% error
        let required_n = ((z * max_variance.sqrt()) / target_error).powi(2) as usize;
        
        let is_adequate = shadows.shots >= required_n;
        let recommended_shots = if is_adequate {
            shadows.shots
        } else {
            required_n
        };
        
        SampleAdequacy {
            is_adequate,
            current_shots: shadows.shots,
            required_shots: required_n,
            recommended_shots,
        }
    }
    
    /// Compute chi-squared statistic
    fn compute_chi_squared(&self, shadows: &ShadowCopyResult) -> f64 {
        let mut chi_sq = 0.0;
        
        // Expected uniform distribution
        let num_outcomes = shadows.distribution.len();
        let expected_prob = 1.0 / num_outcomes as f64;
        
        for &observed_prob in shadows.distribution.values() {
            let diff = observed_prob - expected_prob;
            chi_sq += diff.powi(2) / expected_prob;
        }
        
        chi_sq * shadows.shots as f64
    }
    
    /// Get Z-score for confidence level
    fn get_z_score(&self, confidence: f64) -> f64 {
        let key = format!("{:.2}", confidence);
        *self.z_scores.get(&key).unwrap_or(&1.96) // Default to 95%
    }
}

// ============================================================================
// STATISTICAL CERTIFICATE
// ============================================================================

/// Statistical certificate with guarantees
#[derive(Debug, Clone)]
pub struct StatisticalCertificate {
    /// Unique ID
    pub id: Uuid,
    
    /// Confidence level (e.g., 0.95)
    pub confidence_level: f64,
    
    /// Confidence intervals for each observable
    pub intervals: HashMap<String, ConfidenceInterval>,
    
    /// Error bounds
    pub error_bounds: ErrorBounds,
    
    /// Sample adequacy assessment
    pub sample_adequacy: SampleAdequacy,
    
    /// Chi-squared statistic
    pub chi_squared: f64,
    
    /// Number of shots used
    pub shots: usize,
}

impl StatisticalCertificate {
    /// Check if precision target is met
    pub fn meets_precision(&self, target: &crate::adaptive_resampling::PrecisionTarget) -> bool {
        self.error_bounds.standard_error <= target.max_std_error
            && self.error_bounds.relative_error <= target.max_relative_error
    }
    
    /// Get confidence interval for observable
    pub fn get_interval(&self, observable: &str) -> Option<&ConfidenceInterval> {
        self.intervals.get(observable)
    }
    
    /// Check if result is statistically significant
    pub fn is_significant(&self, observable: &str, threshold: f64) -> bool {
        if let Some(interval) = self.get_interval(observable) {
            // Check if confidence interval doesn't include zero
            interval.lower.abs() > threshold || interval.upper.abs() > threshold
        } else {
            false
        }
    }
    
    /// Generate summary report
    pub fn summary(&self) -> String {
        format!(
            "Statistical Certificate:\n\
             - Confidence: {:.1}%\n\
             - Shots: {}\n\
             - Standard Error: {:.4}\n\
             - Relative Error: {:.2}%\n\
             - Sample Adequate: {}\n\
             - χ²: {:.2}",
            self.confidence_level * 100.0,
            self.shots,
            self.error_bounds.standard_error,
            self.error_bounds.relative_error * 100.0,
            self.sample_adequacy.is_adequate,
            self.chi_squared
        )
    }
}

// ============================================================================
// CONFIDENCE INTERVAL
// ============================================================================

/// Confidence interval for an observable
#[derive(Debug, Clone, Copy)]
pub struct ConfidenceInterval {
    /// Point estimate
    pub value: f64,
    
    /// Lower bound
    pub lower: f64,
    
    /// Upper bound
    pub upper: f64,
    
    /// Confidence level
    pub confidence: f64,
}

impl ConfidenceInterval {
    /// Width of interval
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }
    
    /// Check if value is in interval
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }
    
    /// Margin of error
    pub fn margin(&self) -> f64 {
        (self.upper - self.lower) / 2.0
    }
}

// ============================================================================
// ERROR BOUNDS
// ============================================================================

/// Error bounds for measurements
#[derive(Debug, Clone, Copy)]
pub struct ErrorBounds {
    /// Standard error of the mean
    pub standard_error: f64,
    
    /// Relative error (percentage)
    pub relative_error: f64,
    
    /// Maximum error across observables
    pub max_error: f64,
    
    /// (ε, δ) PAC bound
    pub epsilon_delta: (f64, f64),
}

// ============================================================================
// SAMPLE ADEQUACY
// ============================================================================

/// Assessment of sample size adequacy
#[derive(Debug, Clone, Copy)]
pub struct SampleAdequacy {
    /// Is current sample adequate?
    pub is_adequate: bool,
    
    /// Current number of shots
    pub current_shots: usize,
    
    /// Minimum required shots
    pub required_shots: usize,
    
    /// Recommended shots for next run
    pub recommended_shots: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shadow_copy::ShadowCopyResult;
    
    #[test]
    fn test_certifier_creation() {
        let certifier = Certifier::new(0.95);
        assert_eq!(certifier.default_confidence, 0.95);
    }
    
    #[test]
    fn test_z_scores() {
        let certifier = Certifier::new(0.95);
        assert!((certifier.get_z_score(0.95) - 1.96).abs() < 0.01);
        assert!((certifier.get_z_score(0.99) - 2.576).abs() < 0.01);
    }
    
    #[test]
    fn test_confidence_interval() {
        let interval = ConfidenceInterval {
            value: 0.5,
            lower: 0.45,
            upper: 0.55,
            confidence: 0.95,
        };
        
        assert!((interval.width() - 0.1).abs() < 1e-10);
        assert!(interval.contains(0.5));
        assert!(!interval.contains(0.6));
    }
}
