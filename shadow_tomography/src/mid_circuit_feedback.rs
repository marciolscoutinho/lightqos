// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mid_circuit_feedback.rs — Mid-Circuit Feedback — classical control based on partial measurements
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 21-06-2025
// All rights reserved.
// ---------------------------------------------------------------------------

use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// FEEDBACK HANDLER
// ============================================================================

/// Handler for mid-circuit feedback
#[derive(Debug, Clone)]
pub struct FeedbackHandler {
    /// Feedback rules
    pub rules: Vec<FeedbackRule>,
    
    /// Measurement history
    pub measurement_history: Vec<MeasurementOutcome>,
    
    /// Decision history
    pub decision_history: Vec<FeedbackDecision>,
}

impl FeedbackHandler {
    /// Create new feedback handler
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            measurement_history: Vec::new(),
            decision_history: Vec::new(),
        }
    }
    
    /// Add feedback rule
    pub fn add_rule(&mut self, rule: FeedbackRule) {
        self.rules.push(rule);
    }
    
    /// Process measurement and make feedback decision
    pub fn process_measurement(
        &mut self,
        outcome: MeasurementOutcome,
    ) -> FeedbackDecision {
        // Store measurement
        self.measurement_history.push(outcome.clone());
        
        // Evaluate rules
        let decision = self.evaluate_rules(&outcome);
        
        // Store decision
        self.decision_history.push(decision.clone());
        
        decision
    }
    
    /// Evaluate all rules against measurement outcome
    fn evaluate_rules(&self, outcome: &MeasurementOutcome) -> FeedbackDecision {
        // Try each rule in order
        for rule in &self.rules {
            if rule.condition.evaluate(outcome) {
                return rule.action.clone();
            }
        }
        
        // Default: no action
        FeedbackDecision::NoAction
    }
    
    /// Get last N measurement outcomes
    pub fn recent_outcomes(&self, n: usize) -> Vec<&MeasurementOutcome> {
        let start = self.measurement_history.len().saturating_sub(n);
        self.measurement_history[start..].iter().collect()
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.measurement_history.clear();
        self.decision_history.clear();
    }
}

impl Default for FeedbackHandler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FEEDBACK RULE
// ============================================================================

/// Rule for feedback decision
#[derive(Debug, Clone)]
pub struct FeedbackRule {
    /// Rule ID
    pub id: Uuid,
    
    /// Condition to evaluate
    pub condition: FeedbackCondition,
    
    /// Action to take if condition is true
    pub action: FeedbackDecision,
    
    /// Priority (higher = evaluated first)
    pub priority: i32,
}

impl FeedbackRule {
    /// Create new rule
    pub fn new(condition: FeedbackCondition, action: FeedbackDecision) -> Self {
        Self {
            id: Uuid::new_v4(),
            condition,
            action,
            priority: 0,
        }
    }
    
    /// Set priority
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

// ============================================================================
// FEEDBACK CONDITION
// ============================================================================

/// Condition for triggering feedback
#[derive(Debug, Clone)]
pub enum FeedbackCondition {
    /// Always true
    Always,
    
    /// Qubit measured as |0⟩
    QubitIsZero { qubit: usize },
    
    /// Qubit measured as |1⟩
    QubitIsOne { qubit: usize },
    
    /// Probability above threshold
    ProbabilityAbove { threshold: f64 },
    
    /// Probability below threshold
    ProbabilityBelow { threshold: f64 },
    
    /// Parity check
    ParityCheck { qubits: Vec<usize>, expected: bool },
    
    /// Custom condition
    Custom { predicate: String },
}

impl FeedbackCondition {
    /// Evaluate condition against measurement outcome
    pub fn evaluate(&self, outcome: &MeasurementOutcome) -> bool {
        match self {
            FeedbackCondition::Always => true,
            
            FeedbackCondition::QubitIsZero { qubit } => {
                outcome.get_bit(*qubit) == Some('0')
            }
            
            FeedbackCondition::QubitIsOne { qubit } => {
                outcome.get_bit(*qubit) == Some('1')
            }
            
            FeedbackCondition::ProbabilityAbove { threshold } => {
                outcome.probability > *threshold
            }
            
            FeedbackCondition::ProbabilityBelow { threshold } => {
                outcome.probability < *threshold
            }
            
            FeedbackCondition::ParityCheck { qubits, expected } => {
                let parity = Self::compute_parity(outcome, qubits);
                parity == *expected
            }
            
            FeedbackCondition::Custom { .. } => {
                // Would evaluate custom predicate
                false
            }
        }
    }
    
    /// Compute parity of specified qubits
    fn compute_parity(outcome: &MeasurementOutcome, qubits: &[usize]) -> bool {
        let mut parity = false;
        for &qubit in qubits {
            if let Some('1') = outcome.get_bit(qubit) {
                parity = !parity;
            }
        }
        parity
    }
}

// ============================================================================
// FEEDBACK DECISION
// ============================================================================

/// Decision/action to take based on feedback
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackDecision {
    /// No action
    NoAction,
    
    /// Apply X gate to qubit
    ApplyX { target: usize },
    
    /// Apply Y gate to qubit
    ApplyY { target: usize },
    
    /// Apply Z gate to qubit
    ApplyZ { target: usize },
    
    /// Apply Hadamard gate
    ApplyH { target: usize },
    
    /// Apply CNOT gate
    ApplyCNOT { control: usize, target: usize },
    
    /// Terminate circuit early
    Terminate,
    
    /// Retry measurement
    RetryMeasurement,
    
    /// Custom action
    Custom { action: String },
}

impl FeedbackDecision {
    /// Check if decision modifies circuit
    pub fn modifies_circuit(&self) -> bool {
        !matches!(self, FeedbackDecision::NoAction | FeedbackDecision::Terminate)
    }
}

// ============================================================================
// MEASUREMENT OUTCOME
// ============================================================================

/// Outcome of a mid-circuit measurement
#[derive(Debug, Clone)]
pub struct MeasurementOutcome {
    /// Unique ID
    pub id: Uuid,
    
    /// Bitstring result (e.g., "001")
    pub bitstring: String,
    
    /// Probability of this outcome
    pub probability: f64,
    
    /// Which qubits were measured
    pub measured_qubits: Vec<usize>,
    
    /// Measurement basis
    pub basis: String,
}

impl MeasurementOutcome {
    /// Create new measurement outcome
    pub fn new(bitstring: String, probability: f64, measured_qubits: Vec<usize>) -> Self {
        Self {
            id: Uuid::new_v4(),
            bitstring,
            probability,
            measured_qubits,
            basis: "Z".to_string(),
        }
    }
    
    /// Get bit value for specific qubit
    pub fn get_bit(&self, qubit: usize) -> Option<char> {
        self.bitstring.chars().nth(qubit)
    }
    
    /// Count number of |1⟩ measurements
    pub fn count_ones(&self) -> usize {
        self.bitstring.chars().filter(|&c| c == '1').count()
    }
    
    /// Convert to integer
    pub fn to_int(&self) -> usize {
        usize::from_str_radix(&self.bitstring, 2).unwrap_or(0)
    }
}

// ============================================================================
// MID-CIRCUIT FEEDBACK
// ============================================================================

/// Complete mid-circuit feedback system
#[derive(Debug, Clone)]
pub struct MidCircuitFeedback {
    /// Feedback handler
    pub handler: FeedbackHandler,
    
    /// Enable/disable feedback
    pub enabled: bool,
    
    /// Statistics
    pub stats: FeedbackStatistics,
}

impl MidCircuitFeedback {
    /// Create new mid-circuit feedback system
    pub fn new() -> Self {
        Self {
            handler: FeedbackHandler::new(),
            enabled: true,
            stats: FeedbackStatistics::default(),
        }
    }
    
    /// Process measurement with feedback
    pub fn process(
        &mut self,
        outcome: MeasurementOutcome,
    ) -> Option<FeedbackDecision> {
        if !self.enabled {
            return None;
        }
        
        let decision = self.handler.process_measurement(outcome);
        
        // Update stats
        self.stats.total_measurements += 1;
        if decision != FeedbackDecision::NoAction {
            self.stats.feedback_triggered += 1;
        }
        
        Some(decision)
    }
    
    /// Enable feedback
    pub fn enable(&mut self) {
        self.enabled = true;
    }
    
    /// Disable feedback
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for MidCircuitFeedback {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FEEDBACK STATISTICS
// ============================================================================

/// Statistics for feedback system
#[derive(Debug, Clone, Default)]
pub struct FeedbackStatistics {
    /// Total measurements processed
    pub total_measurements: usize,
    
    /// Number of times feedback was triggered
    pub feedback_triggered: usize,
    
    /// Number of circuit modifications
    pub circuit_modifications: usize,
}

impl FeedbackStatistics {
    /// Feedback trigger rate
    pub fn trigger_rate(&self) -> f64 {
        if self.total_measurements == 0 {
            return 0.0;
        }
        self.feedback_triggered as f64 / self.total_measurements as f64
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_feedback_handler() {
        let handler = FeedbackHandler::new();
        assert_eq!(handler.rules.len(), 0);
        assert_eq!(handler.measurement_history.len(), 0);
    }
    
    #[test]
    fn test_feedback_rule() {
        let condition = FeedbackCondition::QubitIsOne { qubit: 0 };
        let action = FeedbackDecision::ApplyX { target: 1 };
        let rule = FeedbackRule::new(condition, action);
        
        assert_eq!(rule.priority, 0);
    }
    
    #[test]
    fn test_qubit_zero_condition() {
        let condition = FeedbackCondition::QubitIsZero { qubit: 0 };
        let outcome = MeasurementOutcome::new("01".to_string(), 0.5, vec![0, 1]);
        
        assert!(condition.evaluate(&outcome));
    }
    
    #[test]
    fn test_qubit_one_condition() {
        let condition = FeedbackCondition::QubitIsOne { qubit: 1 };
        let outcome = MeasurementOutcome::new("01".to_string(), 0.5, vec![0, 1]);
        
        assert!(condition.evaluate(&outcome));
    }
    
    #[test]
    fn test_parity_check() {
        let condition = FeedbackCondition::ParityCheck {
            qubits: vec![0, 1],
            expected: false, // even parity
        };
        
        let outcome1 = MeasurementOutcome::new("00".to_string(), 0.5, vec![0, 1]);
        assert!(condition.evaluate(&outcome1)); // 0⊕0 = 0 (even)
        
        let outcome2 = MeasurementOutcome::new("11".to_string(), 0.5, vec![0, 1]);
        assert!(condition.evaluate(&outcome2)); // 1⊕1 = 0 (even)
        
        let outcome3 = MeasurementOutcome::new("01".to_string(), 0.5, vec![0, 1]);
        assert!(!condition.evaluate(&outcome3)); // 0⊕1 = 1 (odd)
    }
}
