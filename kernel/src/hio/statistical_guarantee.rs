// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// statistical_guarantee.rs — HIO Statistical Guarantee — confidence certificates (Hoeffding)
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 24-03-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use serde::{Serialize, Deserialize};
use std::f64::consts::E;

// ============================================================================
// PAC GUARANTEE
// ============================================================================

/// PAC guarantee (Probably Approximately Correct)
/// 
/// Guarantees that the estimate is within ε of the true value
/// with probability ≥ 1 - δ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PACGuarantee {
    /// Maximum allowed error (epsilon)
    pub epsilon: f64,
    
    /// Failure probability (delta)
    pub delta: f64,
    
    /// Number of required samples
    pub required_samples: usize,
    
    /// Number of collected samples
    pub collected_samples: usize,
    
    /// Current estimate
    pub current_estimate: Option<f64>,
    
    /// True value (if known)
    pub true_value: Option<f64>,
}

impl PACGuarantee {
    /// Creates a PAC guarantee
    pub fn new(epsilon: f64, delta: f64) -> Self {
        let required_samples = Self::compute_sample_complexity(epsilon, delta);
        
        Self {
            epsilon,
            delta,
            required_samples,
            collected_samples: 0,
            current_estimate: None,
            true_value: None,
        }
    }
    
    /// Calculates the required number of samples
    /// 
    /// Based on Hoeffding: N ≥ ln(2/δ) / (2ε²)
    pub fn compute_sample_complexity(epsilon: f64, delta: f64) -> usize {
        let numerator = (2.0 / delta).ln();
        let denominator = 2.0 * epsilon * epsilon;
        
        (numerator / denominator).ceil() as usize
    }
    
    /// Updates with a new sample
    pub fn add_sample(&mut self, value: f64) {
        self.collected_samples += 1;
        
        if let Some(estimate) = self.current_estimate {
            // Incremental mean
            let n = self.collected_samples as f64;
            self.current_estimate = Some((estimate * (n - 1.0) + value) / n);
        } else {
            self.current_estimate = Some(value);
        }
    }
    
    /// Checks whether the guarantee is satisfied
    pub fn is_satisfied(&self) -> bool {
        self.collected_samples >= self.required_samples
    }
    
    /// Returns confidence interval
    pub fn confidence_interval(&self) -> Option<(f64, f64)> {
        self.current_estimate.map(|est| {
            (est - self.epsilon, est + self.epsilon)
        })
    }
    
    /// Checks whether the estimate is within the expected error
    pub fn within_error(&self) -> Option<bool> {
        if let (Some(estimate), Some(true_val)) = (self.current_estimate, self.true_value) {
            Some((estimate - true_val).abs() <= self.epsilon)
        } else {
            None
        }
    }
}

// ============================================================================
// HOEFFDING BOUND
// ============================================================================

/// Hoeffding bound for sums of random variables
pub struct HoeffdingBound {
    /// Number of variables
    pub num_variables: usize,
    
    /// Range of each variable [a, b]
    pub range: (f64, f64),
    
    /// Confidence (1 - delta)
    pub confidence: f64,
}

impl HoeffdingBound {
    pub fn new(num_variables: usize, range: (f64, f64), confidence: f64) -> Self {
        Self {
            num_variables,
            range,
            confidence,
        }
    }
    
    /// Calculates upper bound
    /// 
    /// P(|X̄ - μ| ≥ t) ≤ 2 exp(-2nt² / (b-a)²)
    pub fn upper_bound(&self, deviation: f64) -> f64 {
        let n = self.num_variables as f64;
        let range_width = self.range.1 - self.range.0;
        
        let exponent = -2.0 * n * deviation * deviation / (range_width * range_width);
        2.0 * E.powf(exponent)
    }
    
    /// Calculates maximum deviation for the specified confidence
    pub fn max_deviation(&self) -> f64 {
        let delta = 1.0 - self.confidence;
        let n = self.num_variables as f64;
        let range_width = self.range.1 - self.range.0;
        
        range_width * ((1.0 / (2.0 * n)) * (2.0 / delta).ln()).sqrt()
    }
}

// ============================================================================
// CHERNOFF BOUND
// ============================================================================

/// Chernoff bound for Bernoulli variables
pub struct ChernoffBound {
    /// Number of trials
    pub num_trials: usize,
    
    /// Expected success probability
    pub expected_prob: f64,
    
    /// Confidence
    pub confidence: f64,
}

impl ChernoffBound {
    pub fn new(num_trials: usize, expected_prob: f64, confidence: f64) -> Self {
        Self {
            num_trials,
            expected_prob,
            confidence,
        }
    }
    
    /// Multiplicative bound
    /// 
    /// P(X ≥ (1+δ)μ) ≤ exp(-μδ²/3)  for δ ∈ [0,1]
    pub fn multiplicative_bound(&self, deviation_factor: f64) -> f64 {
        let mu = self.num_trials as f64 * self.expected_prob;
        let exponent = -mu * deviation_factor * deviation_factor / 3.0;
        E.powf(exponent)
    }
    
    /// Additive bound
    /// 
    /// P(|X - μ| ≥ t) ≤ 2 exp(-2t²/n)
    pub fn additive_bound(&self, deviation: f64) -> f64 {
        let n = self.num_trials as f64;
        let exponent = -2.0 * deviation * deviation / n;
        2.0 * E.powf(exponent)
    }
    
    /// Calculates the required number of trials
    pub fn required_trials(epsilon: f64, delta: f64) -> usize {
        let numerator = (2.0 / delta).ln();
        let denominator = 2.0 * epsilon * epsilon;
        
        (numerator / denominator).ceil() as usize
    }
}

// ============================================================================
// CONFIDENCE INTERVAL
// ============================================================================

/// Statistical confidence interval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Central estimate
    pub estimate: f64,
    
    /// Lower bound
    pub lower_bound: f64,
    
    /// Upper bound
    pub upper_bound: f64,
    
    /// Confidence level (e.g., 0.95 for 95%)
    pub confidence_level: f64,
    
    /// Number of samples
    pub sample_size: usize,
}

impl ConfidenceInterval {
    /// Creates a symmetric confidence interval
    pub fn symmetric(estimate: f64, margin: f64, confidence_level: f64, n: usize) -> Self {
        Self {
            estimate,
            lower_bound: estimate - margin,
            upper_bound: estimate + margin,
            confidence_level,
            sample_size: n,
        }
    }
    
    /// Creates an asymmetric confidence interval
    pub fn asymmetric(
        estimate: f64,
        lower: f64,
        upper: f64,
        confidence_level: f64,
        n: usize,
    ) -> Self {
        Self {
            estimate,
            lower_bound: lower,
            upper_bound: upper,
            confidence_level,
            sample_size: n,
        }
    }
    
    /// Interval width
    pub fn width(&self) -> f64 {
        self.upper_bound - self.lower_bound
    }
    
    /// Checks whether the value lies inside the interval
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower_bound && value <= self.upper_bound
    }
}

// ============================================================================
// GUARANTEED ESTIMATOR
// ============================================================================

/// Statistical estimator with PAC guarantees
pub struct GuaranteedEstimator {
    /// PAC guarantee
    pac_guarantee: PACGuarantee,
    
    /// Collected samples
    samples: Vec<f64>,
    
    /// Current estimateizada
    current_estimate: f64,
    
    /// Sample variance
    sample_variance: f64,
}

impl GuaranteedEstimator {
    pub fn new(epsilon: f64, delta: f64) -> Self {
        Self {
            pac_guarantee: PACGuarantee::new(epsilon, delta),
            samples: Vec::new(),
            current_estimate: 0.0,
            sample_variance: 0.0,
        }
    }
    
    /// Adds a sample
    pub fn add_sample(&mut self, value: f64) {
        self.samples.push(value);
        self.pac_guarantee.add_sample(value);
        self.current_estimate = self.pac_guarantee.current_estimate.unwrap_or(0.0);
        self.update_variance();
    }
    
    fn update_variance(&mut self) {
        if self.samples.len() < 2 {
            self.sample_variance = 0.0;
            return;
        }
        
        let mean = self.current_estimate;
        let sum_sq_diff: f64 = self.samples.iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum();
        
        self.sample_variance = sum_sq_diff / (self.samples.len() - 1) as f64;
    }
    
    /// Returns estimate
    pub fn estimate(&self) -> f64 {
        self.current_estimate
    }
    
    /// Returns confidence interval
    pub fn confidence_interval(&self) -> ConfidenceInterval {
        let margin = self.pac_guarantee.epsilon;
        let confidence = 1.0 - self.pac_guarantee.delta;
        
        ConfidenceInterval::symmetric(
            self.current_estimate,
            margin,
            confidence,
            self.samples.len(),
        )
    }
    
    /// Checks whether the guarantee is satisfied
    pub fn is_converged(&self) -> bool {
        self.pac_guarantee.is_satisfied()
    }
    
    /// Standard error
    pub fn standard_error(&self) -> f64 {
        if self.samples.is_empty() {
            return f64::INFINITY;
        }
        
        (self.sample_variance / self.samples.len() as f64).sqrt()
    }
    
    /// Returns PAC guarantee
    pub fn get_guarantee(&self) -> &PACGuarantee {
        &self.pac_guarantee
    }
}

// ============================================================================
// CONVERGENCE ANALYZER
// ============================================================================

/// Analyzes convergence of estimators
pub struct ConvergenceAnalyzer {
    /// Estimate history
    history: Vec<f64>,
    
    /// Window for analysis
    window_size: usize,
}

impl ConvergenceAnalyzer {
    pub fn new(window_size: usize) -> Self {
        Self {
            history: Vec::new(),
            window_size,
        }
    }
    
    /// Adds estimate
    pub fn add_estimate(&mut self, estimate: f64) {
        self.history.push(estimate);
    }
    
    /// Checks convergence based on window variance
    pub fn has_converged(&self, threshold: f64) -> bool {
        if self.history.len() < self.window_size {
            return false;
        }
        
        let window = &self.history[self.history.len() - self.window_size..];
        
        let mean: f64 = window.iter().sum::<f64>() / window.len() as f64;
        let variance: f64 = window.iter()
            .map(|&x| (x - mean) * (x - mean))
            .sum::<f64>() / window.len() as f64;
        
        variance < threshold
    }
    
    /// Convergence rate (approximate)
    pub fn convergence_rate(&self) -> Option<f64> {
        if self.history.len() < 10 {
            return None;
        }
        
        // Mean difference between consecutive estimates
        let diffs: Vec<f64> = self.history.windows(2)
            .map(|w| (w[1] - w[0]).abs())
            .collect();
        
        Some(diffs.iter().sum::<f64>() / diffs.len() as f64)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pac_guarantee() {
        let pac = PACGuarantee::new(0.1, 0.05);
        
        assert!(pac.epsilon > 0.0);
        assert!(pac.delta > 0.0);
        assert!(pac.required_samples > 0);
    }
    
    #[test]
    fn test_sample_complexity() {
        let epsilon = 0.1;
        let delta = 0.05;
        
        let n = PACGuarantee::compute_sample_complexity(epsilon, delta);
        
        // Should be at least ln(2/0.05) / (2 * 0.1²) ≈ 185
        assert!(n >= 100);
    }
    
    #[test]
    fn test_add_samples() {
        let mut pac = PACGuarantee::new(0.1, 0.05);
        
        for _ in 0..100 {
            pac.add_sample(0.5);
        }
        
        assert_eq!(pac.collected_samples, 100);
        assert!(pac.current_estimate.is_some());
    }
    
    #[test]
    fn test_hoeffding_bound() {
        let bound = HoeffdingBound::new(100, (0.0, 1.0), 0.95);
        
        let prob = bound.upper_bound(0.1);
        assert!(prob > 0.0 && prob < 1.0);
        
        let max_dev = bound.max_deviation();
        assert!(max_dev > 0.0);
    }
    
    #[test]
    fn test_chernoff_bound() {
        let bound = ChernoffBound::new(100, 0.5, 0.95);
        
        let prob = bound.multiplicative_bound(0.2);
        assert!(prob > 0.0 && prob < 1.0);
    }
    
    #[test]
    fn test_confidence_interval() {
        let ci = ConfidenceInterval::symmetric(0.5, 0.1, 0.95, 100);
        
        assert_eq!(ci.lower_bound, 0.4);
        assert_eq!(ci.upper_bound, 0.6);
        assert_eq!(ci.width(), 0.2);
        
        assert!(ci.contains(0.5));
        assert!(ci.contains(0.45));
        assert!(!ci.contains(0.7));
    }
    
    #[test]
    fn test_guaranteed_estimator() {
        let mut estimator = GuaranteedEstimator::new(0.05, 0.01);
        
        // Add samples around 0.7
        for _ in 0..200 {
            estimator.add_sample(0.7);
        }
        
        assert!(estimator.is_converged());
        
        let ci = estimator.confidence_interval();
        assert!(ci.contains(0.7));
    }
    
    #[test]
    fn test_convergence_analyzer() {
        let mut analyzer = ConvergenceAnalyzer::new(10);
        
        // Add estimates converging to 1.0
        for i in 0..50 {
            let estimate = 1.0 - 1.0 / (i + 1) as f64;
            analyzer.add_estimate(estimate);
        }
        
        assert!(analyzer.has_converged(0.01));
    }
}
