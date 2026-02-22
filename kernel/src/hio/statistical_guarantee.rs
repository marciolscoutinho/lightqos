//! Statistical Guarantees (Confidence Certificates)

use super::shadow_copy::ShadowData;
use super::observable_view::MultiBaseViews;
use super::{PrecisionConfig, HIOError};

/// Statistical certificate of a measurement
#[derive(Clone)]
pub struct StatisticalGuarantee {
    pub confidence_level: f64,
    pub standard_error: f64,
    pub confidence_interval: (f64, f64),
    pub samples_used: usize,
    pub meets_requirements: bool,
}

/// Computes statistical guarantees
pub fn compute_guarantee(
    shadows: &ShadowData,
    views: &MultiBaseViews,
    config: &PrecisionConfig,
) -> Result<StatisticalGuarantee, HIOError> {
    // Checks minimum sample count
    if shadows.total_samples < config.min_samples {
        return Err(HIOError::InsufficientSamples);
    }
    
    // Computes standard error (approximation)
    let standard_error = (shadows.entropy / (shadows.total_samples as f64).sqrt())
        .max(1.0 / (shadows.total_samples as f64).sqrt());
    
    // Confidence interval (assuming a normal distribution)
    let z_score = match config.confidence_level {
        x if x >= 0.99 => 2.576,
        x if x >= 0.95 => 1.96,
        x if x >= 0.90 => 1.645,
        _ => 1.96,
    };
    
    let margin = z_score * standard_error;
    let confidence_interval = (
        0.5 - margin, // Simplified central expectation
        0.5 + margin,
    );
    
    // Checks whether it meets requirements
    let meets_requirements = standard_error <= config.max_error;
    
    Ok(StatisticalGuarantee {
        confidence_level: config.confidence_level,
        standard_error,
        confidence_interval,
        samples_used: shadows.total_samples,
        meets_requirements,
    })
}

/// Computes the number of samples required to reach a target precision
pub fn required_samples(target_error: f64, confidence: f64) -> usize {
    let z_score = match confidence {
        x if x >= 0.99 => 2.576,
        x if x >= 0.95 => 1.96,
        _ => 1.645,
    };
    
    // n = (z * σ / E)²
    // Assuming σ ≈ 0.5 (worst case for a binomial distribution)
    let n = ((z_score * 0.5) / target_error).powi(2);
    n.ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_required_samples() {
        let n = required_samples(0.01, 0.95);
        assert!(n > 9000); // ~9604 for 95% confidence, 1% error
    }
}
