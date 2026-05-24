// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// contract.rs — TLM Contract — temporal SLA contracts with deadlines and priorities
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 08-09-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

// ============================================================================
// CONTRACT TYPES
// ============================================================================

/// SLA contract type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractType {
    /// Temporal contract (timing)
    Temporal,
    
    /// Fidelity contract (quality)
    Fidelity,
    
    /// Coherence contract (phase)
    Coherence,
    
    /// Bandwidth contract (throughput)
    Bandwidth,
}

/// Contract severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ContractSeverity {
    /// Critical - failure is unacceptable
    Critical = 4,
    
    /// High - requires immediate action
    High = 3,
    
    /// Medium - requires warning
    Medium = 2,
    
    /// Low - best effort
    Low = 1,
}

// ============================================================================
// TEMPORAL CONTRACT
// ============================================================================

/// Temporal contract - defines timing guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalContract {
    /// Unique contract ID
    pub id: Uuid,
    
    /// Contract name
    pub name: String,
    
    /// Tipo: Strict, Flexible, Relaxed
    pub variant: TemporalVariant,
    
    /// Maximum allowed duration (nanoseconds)
    pub max_duration_ns: u64,
    
    /// Tolerance (±nanoseconds)
    pub tolerance_ns: u64,
    
    /// Absolute deadline (optional)
    pub deadline: Option<Instant>,
    
    /// Scheduler priority
    pub priority: u8,
    
    /// Severity
    pub severity: ContractSeverity,
    
    /// Phase window (radians)
    pub phase_window_rad: Option<f64>,
    
    /// Allow rollback in case of violation?
    pub allow_rollback: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemporalVariant {
    /// Strict: no tolerance, maximum priority
    Strict,
    
    /// Flexible: with tolerance, medium priority
    Flexible,
    
    /// Relaxed: high tolerance, low priority
    Relaxed,
}

impl TemporalContract {
    /// Creates a Strict contract (no tolerance)
    pub fn strict() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "strict_timing".to_string(),
            variant: TemporalVariant::Strict,
            max_duration_ns: 100,  // 100ns default
            tolerance_ns: 0,
            deadline: None,
            priority: 255,  // Maximum priority
            severity: ContractSeverity::Critical,
            phase_window_rad: Some(0.01),  // ±0.01 rad
            allow_rollback: true,
        }
    }
    
    /// Creates a Flexible contract (with tolerance)
    pub fn flexible(tolerance_ns: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "flexible_timing".to_string(),
            variant: TemporalVariant::Flexible,
            max_duration_ns: 500,
            tolerance_ns,
            deadline: None,
            priority: 128,
            severity: ContractSeverity::Medium,
            phase_window_rad: Some(0.1),
            allow_rollback: true,
        }
    }
    
    /// Creates a Relaxed contract (high tolerance)
    pub fn relaxed(tolerance_ns: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "relaxed_timing".to_string(),
            variant: TemporalVariant::Relaxed,
            max_duration_ns: 2000,
            tolerance_ns,
            deadline: None,
            priority: 50,
            severity: ContractSeverity::Low,
            phase_window_rad: Some(0.5),
            allow_rollback: false,
        }
    }
    
    /// Validates whether a duration satisfies the contract
    pub fn validate_duration(&self, actual_duration_ns: u64) -> ValidationResult {
        let upper_bound = self.max_duration_ns + self.tolerance_ns;
        
        if actual_duration_ns <= upper_bound {
            ValidationResult::Success
        } else {
            let violation_ns = actual_duration_ns - upper_bound;
            ValidationResult::Violation {
                contract_id: self.id,
                violation_type: ViolationType::TimingExceeded,
                severity: self.severity,
                details: format!(
                    "Duration {} ns exceeds limit {} ns (violation: {} ns)",
                    actual_duration_ns,
                    upper_bound,
                    violation_ns
                ),
            }
        }
    }
    
    /// Validates phase
    pub fn validate_phase(&self, actual_phase_rad: f64, expected_phase_rad: f64) -> ValidationResult {
        if let Some(window) = self.phase_window_rad {
            let phase_error = (actual_phase_rad - expected_phase_rad).abs();
            
            if phase_error <= window {
                ValidationResult::Success
            } else {
                ValidationResult::Violation {
                    contract_id: self.id,
                    violation_type: ViolationType::PhaseCoherence,
                    severity: self.severity,
                    details: format!(
                        "Phase error {:.4} rad exceeds window {:.4} rad",
                        phase_error,
                        window
                    ),
                }
            }
        } else {
            ValidationResult::Success
        }
    }
}

// ============================================================================
// FIDELITY CONTRACT
// ============================================================================

/// Fidelity contract - defines quality guarantees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FidelityContract {
    /// Unique ID
    pub id: Uuid,
    
    /// Nome
    pub name: String,
    
    /// Minimum required fidelity [0, 1]
    pub min_fidelity: f64,
    
    /// Maximum number of attempts (retries)
    pub max_retries: usize,
    
    /// Automatic optimization if it fails?
    pub auto_optimize: bool,
    
    /// Severity
    pub severity: ContractSeverity,
    
    /// Fidelity metric
    pub metric: FidelityMetric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FidelityMetric {
    /// State fidelity
    State,
    
    /// Process fidelity
    Process,
    
    /// Gate fidelity
    Gate,
    
    /// Entanglement fidelity
    Entanglement,
}

impl FidelityContract {
    /// Creates a fidelity contract
    pub fn new(min_fidelity: f64, max_retries: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: "fidelity_guarantee".to_string(),
            min_fidelity: min_fidelity.clamp(0.0, 1.0),
            max_retries,
            auto_optimize: true,
            severity: ContractSeverity::High,
            metric: FidelityMetric::State,
        }
    }
    
    /// Validates fidelity
    pub fn validate_fidelity(&self, actual_fidelity: f64) -> ValidationResult {
        if actual_fidelity >= self.min_fidelity {
            ValidationResult::Success
        } else {
            ValidationResult::Violation {
                contract_id: self.id,
                violation_type: ViolationType::FidelityBelowThreshold,
                severity: self.severity,
                details: format!(
                    "Fidelity {:.4} below minimum {:.4} (deficit: {:.4})",
                    actual_fidelity,
                    self.min_fidelity,
                    self.min_fidelity - actual_fidelity
                ),
            }
        }
    }
}

// ============================================================================
// COHERENCE CONTRACT
// ============================================================================

/// Coherence contract - maintains global phase coherence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoherenceContract {
    /// Unique ID
    pub id: Uuid,
    
    /// Minimum coherence time (microseconds)
    pub min_coherence_time_us: f64,
    
    /// Covered qubits
    pub qubits: Vec<usize>,
    
    /// Allow dynamic correction?
    pub allow_dynamic_correction: bool,
    
    /// Severity
    pub severity: ContractSeverity,
}

impl CoherenceContract {
    pub fn new(min_coherence_time_us: f64, qubits: Vec<usize>) -> Self {
        Self {
            id: Uuid::new_v4(),
            min_coherence_time_us,
            qubits,
            allow_dynamic_correction: true,
            severity: ContractSeverity::High,
        }
    }
    
    /// Validates coherence time
    pub fn validate_coherence(&self, actual_coherence_us: f64) -> ValidationResult {
        if actual_coherence_us >= self.min_coherence_time_us {
            ValidationResult::Success
        } else {
            ValidationResult::Violation {
                contract_id: self.id,
                violation_type: ViolationType::CoherenceLoss,
                severity: self.severity,
                details: format!(
                    "Coherence {:.2} μs below minimum {:.2} μs",
                    actual_coherence_us,
                    self.min_coherence_time_us
                ),
            }
        }
    }
}

// ============================================================================
// BANDWIDTH CONTRACT
// ============================================================================

/// Bandwidth contract (throughput)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BandwidthContract {
    /// Unique ID
    pub id: Uuid,
    
    /// Quantum operations per second (minimum)
    pub min_ops_per_second: f64,
    
    /// Entanglement rate (EPR pairs/second)
    pub min_epr_rate: Option<f64>,
    
    /// Severity
    pub severity: ContractSeverity,
}

impl BandwidthContract {
    pub fn new(min_ops_per_second: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            min_ops_per_second,
            min_epr_rate: None,
            severity: ContractSeverity::Medium,
        }
    }
    
    /// Validates throughput
    pub fn validate_throughput(&self, actual_ops_per_second: f64) -> ValidationResult {
        if actual_ops_per_second >= self.min_ops_per_second {
            ValidationResult::Success
        } else {
            ValidationResult::Violation {
                contract_id: self.id,
                violation_type: ViolationType::ThroughputBelowTarget,
                severity: self.severity,
                details: format!(
                    "Throughput {:.1} ops/s below target {:.1} ops/s",
                    actual_ops_per_second,
                    self.min_ops_per_second
                ),
            }
        }
    }
}

// ============================================================================
// VALIDATION RESULT
// ============================================================================

/// Contract validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationResult {
    /// Contract fulfilled
    Success,
    
    /// Contract violated
    Violation {
        contract_id: Uuid,
        violation_type: ViolationType,
        severity: ContractSeverity,
        details: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationType {
    TimingExceeded,
    FidelityBelowThreshold,
    PhaseCoherence,
    CoherenceLoss,
    ThroughputBelowTarget,
}

impl ValidationResult {
    /// Checks whether it succeeded
    pub fn is_success(&self) -> bool {
        matches!(self, ValidationResult::Success)
    }
    
    /// Gets severity (None if successful)
    pub fn severity(&self) -> Option<ContractSeverity> {
        match self {
            ValidationResult::Violation { severity, .. } => Some(*severity),
            _ => None,
        }
    }
}

// ============================================================================
// CONTRACT MANAGER
// ============================================================================

/// Central contract manager
pub struct ContractManager {
    /// Active temporal contracts
    temporal_contracts: HashMap<Uuid, TemporalContract>,
    
    /// Active fidelity contracts
    fidelity_contracts: HashMap<Uuid, FidelityContract>,
    
    /// Active coherence contracts
    coherence_contracts: HashMap<Uuid, CoherenceContract>,
    
    /// Active bandwidth contracts
    bandwidth_contracts: HashMap<Uuid, BandwidthContract>,
    
    /// Violation history
    violation_history: Vec<ViolationRecord>,
    
    /// Statistics
    stats: ContractStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViolationRecord {
    pub timestamp: Instant,
    pub contract_id: Uuid,
    pub contract_type: ContractType,
    pub violation_type: ViolationType,
    pub severity: ContractSeverity,
    pub details: String,
    pub action_taken: ViolationAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationAction {
    /// No action (logged only)
    None,
    
    /// Rollback to snapshot
    Rollback,
    
    /// Retry operation
    Retry,
    
    /// Optimization applied
    Optimize,
    
    /// Operation aborted
    Abort,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractStatistics {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub total_violations: u64,
    pub critical_violations: u64,
    pub rollbacks_triggered: u64,
    pub retries_attempted: u64,
}

impl ContractManager {
    pub fn new() -> Self {
        Self {
            temporal_contracts: HashMap::new(),
            fidelity_contracts: HashMap::new(),
            coherence_contracts: HashMap::new(),
            bandwidth_contracts: HashMap::new(),
            violation_history: Vec::new(),
            stats: ContractStatistics::default(),
        }
    }
    
    // ========================================================================
    // CONTRACT REGISTRATION
    // ========================================================================
    
    /// Registers temporal contract
    pub fn register_temporal(&mut self, contract: TemporalContract) -> Uuid {
        let id = contract.id;
        self.temporal_contracts.insert(id, contract);
        id
    }
    
    /// Registers fidelity contract
    pub fn register_fidelity(&mut self, contract: FidelityContract) -> Uuid {
        let id = contract.id;
        self.fidelity_contracts.insert(id, contract);
        id
    }
    
    /// Registers coherence contract
    pub fn register_coherence(&mut self, contract: CoherenceContract) -> Uuid {
        let id = contract.id;
        self.coherence_contracts.insert(id, contract);
        id
    }
    
    /// Registers bandwidth contract
    pub fn register_bandwidth(&mut self, contract: BandwidthContract) -> Uuid {
        let id = contract.id;
        self.bandwidth_contracts.insert(id, contract);
        id
    }
    
    // ========================================================================
    // VALIDATION
    // ========================================================================
    
    /// Validates all temporal contracts for an operation
    pub fn validate_temporal_all(
        &mut self,
        duration_ns: u64,
        phase_rad: Option<f64>,
    ) -> Vec<ValidationResult> {
        self.stats.total_validations += self.temporal_contracts.len() as u64;
        
        let mut results = Vec::new();
        
        for contract in self.temporal_contracts.values() {
            // Validate duration
            let result = contract.validate_duration(duration_ns);
            
            if !result.is_success() {
                self.handle_violation(
                    ContractType::Temporal,
                    &result,
                    contract.allow_rollback,
                );
            } else {
                self.stats.successful_validations += 1;
            }
            
            results.push(result);
            
            // Validate phase (if applicable)
            if let Some(actual_phase) = phase_rad {
                let phase_result = contract.validate_phase(actual_phase, 0.0);
                
                if !phase_result.is_success() {
                    self.handle_violation(
                        ContractType::Temporal,
                        &phase_result,
                        contract.allow_rollback,
                    );
                }
                
                results.push(phase_result);
            }
        }
        
        results
    }
    
    /// Validates a specific fidelity contract
    pub fn validate_fidelity(
        &mut self,
        contract_id: Uuid,
        actual_fidelity: f64,
    ) -> ValidationResult {
        self.stats.total_validations += 1;
        
        if let Some(contract) = self.fidelity_contracts.get(&contract_id) {
            let result = contract.validate_fidelity(actual_fidelity);
            
            if !result.is_success() {
                self.handle_violation(
                    ContractType::Fidelity,
                    &result,
                    contract.auto_optimize,
                );
            } else {
                self.stats.successful_validations += 1;
            }
            
            result
        } else {
            ValidationResult::Success  // Contract not found = no restriction
        }
    }
    
    // ========================================================================
    // VIOLATION HANDLING
    // ========================================================================
    
    fn handle_violation(
        &mut self,
        contract_type: ContractType,
        result: &ValidationResult,
        allow_corrective_action: bool,
    ) {
        if let ValidationResult::Violation {
            contract_id,
            violation_type,
            severity,
            details,
        } = result
        {
            self.stats.total_violations += 1;
            
            if *severity == ContractSeverity::Critical {
                self.stats.critical_violations += 1;
            }
            
            // Decide action
            let action = if allow_corrective_action {
                match violation_type {
                    ViolationType::TimingExceeded => {
                        if *severity >= ContractSeverity::High {
                            self.stats.rollbacks_triggered += 1;
                            ViolationAction::Rollback
                        } else {
                            ViolationAction::None
                        }
                    }
                    ViolationType::FidelityBelowThreshold => {
                        self.stats.retries_attempted += 1;
                        ViolationAction::Retry
                    }
                    _ => ViolationAction::None,
                }
            } else {
                ViolationAction::None
            };
            
            // Register violation
            let record = ViolationRecord {
                timestamp: Instant::now(),
                contract_id: *contract_id,
                contract_type,
                violation_type: *violation_type,
                severity: *severity,
                details: details.clone(),
                action_taken: action,
            };
            
            self.violation_history.push(record);
        }
    }
    
    // ========================================================================
    // QUERIES
    // ========================================================================
    
    /// Gets statistics
    pub fn get_statistics(&self) -> &ContractStatistics {
        &self.stats
    }
    
    /// Gets recent violations
    pub fn get_recent_violations(&self, limit: usize) -> &[ViolationRecord] {
        let len = self.violation_history.len();
        if len <= limit {
            &self.violation_history
        } else {
            &self.violation_history[len - limit..]
        }
    }
    
    /// Gets success rate
    pub fn success_rate(&self) -> f64 {
        if self.stats.total_validations == 0 {
            1.0
        } else {
            self.stats.successful_validations as f64 / self.stats.total_validations as f64
        }
    }
    
    /// Clears expired contracts
    pub fn cleanup_expired(&mut self) {
        // TODO: Implement timestamp-based expiration logic
    }
}

impl Default for ContractManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_temporal_contract_strict() {
        let contract = TemporalContract::strict();
        
        assert_eq!(contract.variant, TemporalVariant::Strict);
        assert_eq!(contract.tolerance_ns, 0);
        assert_eq!(contract.priority, 255);
        
        // Validate duration within the limit
        let result = contract.validate_duration(50);
        assert!(result.is_success());
        
        // Validate exceeded duration
        let result = contract.validate_duration(200);
        assert!(!result.is_success());
    }
    
    #[test]
    fn test_fidelity_contract() {
        let contract = FidelityContract::new(0.95, 5);
        
        // Fidelity above the minimum
        let result = contract.validate_fidelity(0.97);
        assert!(result.is_success());
        
        // Fidelity below the minimum
        let result = contract.validate_fidelity(0.90);
        assert!(!result.is_success());
        assert_eq!(
            result.severity(),
            Some(ContractSeverity::High)
        );
    }
    
    #[test]
    fn test_contract_manager() {
        let mut manager = ContractManager::new();
        
        // Register contract
        let contract = TemporalContract::strict();
        let id = manager.register_temporal(contract);
        
        assert_eq!(manager.temporal_contracts.len(), 1);
        
        // Validate
        let results = manager.validate_temporal_all(50, None);
        assert_eq!(results.len(), 1);
        assert!(results[0].is_success());
        
        // Statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.total_validations, 1);
        assert_eq!(stats.successful_validations, 1);
        assert_eq!(stats.total_violations, 0);
    }
    
    #[test]
    fn test_violation_handling() {
        let mut manager = ContractManager::new();
        
        let contract = TemporalContract::strict();
        manager.register_temporal(contract);
        
        // Cause violation
        let results = manager.validate_temporal_all(500, None);
        
        assert!(!results[0].is_success());
        assert_eq!(manager.stats.total_violations, 1);
        assert_eq!(manager.violation_history.len(), 1);
    }
    
    #[test]
    fn test_success_rate() {
        let mut manager = ContractManager::new();
        
        manager.stats.total_validations = 100;
        manager.stats.successful_validations = 95;
        
        let rate = manager.success_rate();
        assert!((rate - 0.95).abs() < 0.001);
    }
}
