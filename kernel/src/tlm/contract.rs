//! Temporal Contracts (Service Level Agreements)

use std::time::Duration;

/// Temporal contract for quantum operations
#[derive(Clone)]
pub struct TemporalContract {
    /// Maximum allowed latency
    pub max_latency: Duration,
    
    /// Phase deadline (fraction of the period)
    pub deadline_phase: f64,
    
    /// Whether to rollback on violation
    pub rollback_on_violation: bool,
    
    /// Number of retries before failing
    pub max_retries: u32,
}

impl TemporalContract {
    /// Creates a default (permissive) contract
    pub fn default_permissive() -> Self {
        TemporalContract {
            max_latency: Duration::from_millis(10),
            deadline_phase: 0.5,
            rollback_on_violation: false,
            max_retries: 1,
        }
    }
    
    /// Creates a strict contract (high performance)
    pub fn strict() -> Self {
        TemporalContract {
            max_latency: Duration::from_nanos(100),
            deadline_phase: 0.05,
            rollback_on_violation: true,
            max_retries: 3,
        }
    }
    
    /// Validates the contract
    pub fn validate(&self) -> Result<(), super::TLMError> {
        if self.max_latency.is_zero() {
            return Err(super::TLMError::InvalidContract);
        }
        
        if self.deadline_phase <= 0.0 || self.deadline_phase > 1.0 {
            return Err(super::TLMError::InvalidContract);
        }
        
        Ok(())
    }
    
    /// Checks whether the measured latency violates the contract
    pub fn is_violated_by(&self, measured_latency: Duration) -> bool {
        measured_latency > self.max_latency
    }
    
    /// Computes the remaining time margin
    pub fn remaining_margin(&self, elapsed: Duration) -> Duration {
        self.max_latency.saturating_sub(elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_contract_validation() {
        let contract = TemporalContract::strict();
        assert!(contract.validate().is_ok());
        
        let invalid = TemporalContract {
            max_latency: Duration::ZERO,
            deadline_phase: 0.1,
            rollback_on_violation: false,
            max_retries: 1,
        };
        assert!(invalid.validate().is_err());
    }
    
    #[test]
    fn test_violation_detection() {
        let contract = TemporalContract {
            max_latency: Duration::from_nanos(100),
            deadline_phase: 0.1,
            rollback_on_violation: true,
            max_retries: 1,
        };
        
        assert!(!contract.is_violated_by(Duration::from_nanos(50)));
        assert!(contract.is_violated_by(Duration::from_nanos(150)));
    }
}
