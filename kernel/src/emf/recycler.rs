// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// recycler.rs — EMF Recycler — degraded pair garbage collection and recycling
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 30-11-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::emf::metrics::{
    EntangledPairState,
    ThermodynamicPhase,
    ThermodynamicPhaseClassifier,
    EntanglementMetricsCalculator,
};

// ============================================================================
// RECYCLING POLICY
// ============================================================================

/// Recycling policy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecyclingPolicy {
    /// Conservative: recycle only pairs in the Radiation phase
    Conservative,
    
    /// Balanced: recycle Radiation and Degradation
    Balanced,
    
    /// Aggressive: recycle any pair with fidelity < threshold
    Aggressive { threshold: u8 },  // threshold as a percentage (0-100)
    
    /// Adaptive: adjust based on pool pressure
    Adaptive,
}

impl RecyclingPolicy {
    /// Checks whether a pair should be recycled
    pub fn should_recycle(
        &self,
        state: &EntangledPairState,
        pool_pressure: f64,
    ) -> bool {
        let phase = ThermodynamicPhaseClassifier::classify_phase(state);
        
        match self {
            RecyclingPolicy::Conservative => {
                matches!(phase, ThermodynamicPhase::Radiation)
            }
            
            RecyclingPolicy::Balanced => {
                matches!(
                    phase,
                    ThermodynamicPhase::Radiation | ThermodynamicPhase::Degradation
                )
            }
            
            RecyclingPolicy::Aggressive { threshold } => {
                let threshold_f64 = (*threshold as f64) / 100.0;
                state.fidelity < threshold_f64
            }
            
            RecyclingPolicy::Adaptive => {
                // If pressure is high (>80%), recycle aggressively
                if pool_pressure > 0.8 {
                    state.fidelity < 0.8
                } else if pool_pressure > 0.5 {
                    matches!(
                        phase,
                        ThermodynamicPhase::Radiation | ThermodynamicPhase::Degradation
                    )
                } else {
                    matches!(phase, ThermodynamicPhase::Radiation)
                }
            }
        }
    }
}

// ============================================================================
// RECYCLING STRATEGY
// ============================================================================

/// Resource extraction strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecyclingStrategy {
    /// Residual ergotropy extraction
    ErgotropyExtraction,
    
    /// Purification (combine degraded pairs to improve fidelity)
    Purification,
    
    /// Distillation (extract entanglement from multiple pairs)
    Distillation,
    
    /// Simple discard (no extraction)
    Discard,
}

// ============================================================================
// RECYCLING RESULT
// ============================================================================

/// Result of a recycling operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecyclingResult {
    /// Recycled pairs
    pub recycled_pairs: Vec<Uuid>,
    
    /// Total ergotropy extraída
    pub extracted_ergotropy: f64,
    
    /// Dissipated entropy
    pub dissipated_entropy: f64,
    
    /// New pairs generated (if applicable)
    pub generated_pairs: Vec<Uuid>,
    
    /// Strategy used
    pub strategy: RecyclingStrategy,
    
    /// Processing time (ns)
    pub processing_time_ns: u64,
    
    /// Success
    pub success: bool,
}

// ============================================================================
// RECYCLER
// ============================================================================

/// Entangled-pair recycler
pub struct EntanglementRecycler {
    /// Recycling policy
    policy: RecyclingPolicy,
    
    /// Default strategy
    default_strategy: RecyclingStrategy,
    
    /// Queue of pairs to recycle
    recycling_queue: Arc<RwLock<VecDeque<Uuid>>>,
    
    /// Statistics
    stats: RecyclingStatistics,
    
    /// Settings
    config: RecyclerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecyclerConfig {
    /// Scan interval (ms)
    pub scan_interval_ms: u64,
    
    /// Maximum queue size
    pub max_queue_size: usize,
    
    /// Ergotropy threshold for extraction
    pub min_ergotropy_extraction: f64,
    
    /// Enable purification?
    pub enable_purification: bool,
    
    /// Minimum number of pairs for distillation
    pub min_pairs_for_distillation: usize,
}

impl Default for RecyclerConfig {
    fn default() -> Self {
        Self {
            scan_interval_ms: 1000,  // 1 segundo
            max_queue_size: 1000,
            min_ergotropy_extraction: 0.1,
            enable_purification: true,
            min_pairs_for_distillation: 3,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecyclingStatistics {
    /// Total recycled pairs
    pub total_recycled: u64,
    
    /// Total extracted ergotropy
    pub total_ergotropy_extracted: f64,
    
    /// Total dissipated entropy
    pub total_entropy_dissipated: f64,
    
    /// Pairs generated by recycling
    pub pairs_generated: u64,
    
    /// Purifications performed
    pub purifications_performed: u64,
    
    /// Distillations performed
    pub distillations_performed: u64,
    
    /// Success rate
    pub success_rate: f64,
}

impl EntanglementRecycler {
    pub fn new(policy: RecyclingPolicy, strategy: RecyclingStrategy) -> Self {
        Self {
            policy,
            default_strategy: strategy,
            recycling_queue: Arc::new(RwLock::new(VecDeque::new())),
            stats: RecyclingStatistics::default(),
            config: RecyclerConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: RecyclerConfig) -> Self {
        self.config = config;
        self
    }
    
    // ========================================================================
    // QUEUEING
    // ========================================================================
    
    /// Adds pair to the recycling queue
    pub fn enqueue_for_recycling(&mut self, pair_id: Uuid) -> Result<(), String> {
        let mut queue = self.recycling_queue.write().unwrap();
        
        if queue.len() >= self.config.max_queue_size {
            return Err("Recycling queue is full".to_string());
        }
        
        queue.push_back(pair_id);
        Ok(())
    }
    
    /// Scans pool and enqueues candidate pairs
    pub fn scan_and_enqueue(
        &mut self,
        pool_pairs: &HashMap<Uuid, EntangledPairState>,
        pool_pressure: f64,
    ) -> usize {
        let mut enqueued = 0;
        
        for (pair_id, state) in pool_pairs {
            if self.policy.should_recycle(state, pool_pressure) {
                if self.enqueue_for_recycling(*pair_id).is_ok() {
                    enqueued += 1;
                }
            }
        }
        
        enqueued
    }
    
    // ========================================================================
    // RECYCLING
    // ========================================================================
    
    /// Processes recycling queue
    pub fn process_queue(
        &mut self,
        pool_pairs: &mut HashMap<Uuid, EntangledPairState>,
    ) -> Vec<RecyclingResult> {
        let mut results = Vec::new();
        let mut queue = self.recycling_queue.write().unwrap();
        
        while let Some(pair_id) = queue.pop_front() {
            if let Some(state) = pool_pairs.get(&pair_id) {
                let result = self.recycle_pair(pair_id, state, pool_pairs);
                
                if result.success {
                    // Remove recycled pair from the pool
                    pool_pairs.remove(&pair_id);
                    
                    // Add newly generated pairs
                    for new_pair_id in &result.generated_pairs {
                        // TODO: Create new state for generated pairs
                    }
                }
                
                results.push(result);
            }
        }
        
        results
    }
    
    /// Recycles an individual pair
    fn recycle_pair(
        &mut self,
        pair_id: Uuid,
        state: &EntangledPairState,
        pool_pairs: &HashMap<Uuid, EntangledPairState>,
    ) -> RecyclingResult {
        let start_time = Self::current_time_ns();
        
        let result = match self.default_strategy {
            RecyclingStrategy::ErgotropyExtraction => {
                self.extract_ergotropy(pair_id, state)
            }
            
            RecyclingStrategy::Purification => {
                if self.config.enable_purification {
                    self.purify_pairs(pair_id, state, pool_pairs)
                } else {
                    self.extract_ergotropy(pair_id, state)
                }
            }
            
            RecyclingStrategy::Distillation => {
                self.distill_entanglement(pair_id, state, pool_pairs)
            }
            
            RecyclingStrategy::Discard => {
                self.discard_pair(pair_id, state)
            }
        };
        
        let processing_time = Self::current_time_ns() - start_time;
        
        // Update statistics
        self.update_stats(&result);
        
        RecyclingResult {
            processing_time_ns: processing_time,
            ..result
        }
    }
    
    // ========================================================================
    // ESTRATÉGIAS DE RECYCLING
    // ========================================================================
    
    /// Ergotropy extraction
    fn extract_ergotropy(
        &self,
        pair_id: Uuid,
        state: &EntangledPairState,
    ) -> RecyclingResult {
        // Extract residual ergotropy from the pair
        let extracted_ergotropy = if state.ergotropy >= self.config.min_ergotropy_extraction {
            state.ergotropy
        } else {
            0.0
        };
        
        let dissipated_entropy = state.entanglement_entropy;
        
        RecyclingResult {
            recycled_pairs: vec![pair_id],
            extracted_ergotropy,
            dissipated_entropy,
            generated_pairs: Vec::new(),
            strategy: RecyclingStrategy::ErgotropyExtraction,
            processing_time_ns: 0,
            success: true,
        }
    }
    
    /// Pair purification
    /// Combines two degraded pairs to create a pair with higher fidelity
    fn purify_pairs(
        &mut self,
        pair_id: Uuid,
        state: &EntangledPairState,
        pool_pairs: &HashMap<Uuid, EntangledPairState>,
    ) -> RecyclingResult {
        // Find another degraded pair for purification
        let partner_pair = pool_pairs
            .iter()
            .filter(|(id, s)| {
                **id != pair_id &&
                s.fidelity < 0.9 &&
                s.fidelity > 0.5 &&
                ThermodynamicPhaseClassifier::classify_phase(s) == ThermodynamicPhase::Degradation
            })
            .next();
        
        if let Some((partner_id, partner_state)) = partner_pair {
            // Purification protocol: F_new ≈ F1² + F2² - 2F1²F2²
            let f1 = state.fidelity;
            let f2 = partner_state.fidelity;
            let purified_fidelity = f1 * f1 + f2 * f2 - 2.0 * f1 * f1 * f2 * f2;
            
            // Combined ergotropy (part is lost in the process)
            let combined_ergotropy = (state.ergotropy + partner_state.ergotropy) * 0.5;
            
            // Dissipated entropy
            let dissipated_entropy = state.entanglement_entropy + partner_state.entanglement_entropy;
            
            // Generate new purified pair
            let new_pair_id = Uuid::new_v4();
            
            self.stats.purifications_performed += 1;
            
            RecyclingResult {
                recycled_pairs: vec![pair_id, *partner_id],
                extracted_ergotropy: combined_ergotropy,
                dissipated_entropy,
                generated_pairs: vec![new_pair_id],
                strategy: RecyclingStrategy::Purification,
                processing_time_ns: 0,
                success: purified_fidelity > state.fidelity,
            }
        } else {
            // No pair available for purification; perform simple extraction
            self.extract_ergotropy(pair_id, state)
        }
    }
    
    /// Entanglement distillation
    /// Combines multiple pairs to create a high-fidelity pair
    fn distill_entanglement(
        &mut self,
        pair_id: Uuid,
        state: &EntangledPairState,
        pool_pairs: &HashMap<Uuid, EntangledPairState>,
    ) -> RecyclingResult {
        // Collect pairs for distillation
        let mut candidate_pairs: Vec<(Uuid, &EntangledPairState)> = pool_pairs
            .iter()
            .filter(|(id, s)| {
                **id != pair_id &&
                s.fidelity > 0.5 &&
                s.fidelity < 0.85
            })
            .map(|(id, s)| (*id, s))
            .take(self.config.min_pairs_for_distillation - 1)
            .collect();
        
        candidate_pairs.insert(0, (pair_id, state));
        
        if candidate_pairs.len() >= self.config.min_pairs_for_distillation {
            // Distillation protocol
            // Fidelity increases, but multiple pairs are consumed
            
            let avg_fidelity: f64 = candidate_pairs.iter()
                .map(|(_, s)| s.fidelity)
                .sum::<f64>() / candidate_pairs.len() as f64;
            
            // Distillation improves fidelity
            let distilled_fidelity = (avg_fidelity + 0.2).min(0.99);
            
            // Combined ergotropy
            let total_ergotropy: f64 = candidate_pairs.iter()
                .map(|(_, s)| s.ergotropy)
                .sum();
            
            let extracted_ergotropy = total_ergotropy * 0.3;  // 30% extracted
            
            // Total entropy
            let total_entropy: f64 = candidate_pairs.iter()
                .map(|(_, s)| s.entanglement_entropy)
                .sum();
            
            let recycled_ids: Vec<Uuid> = candidate_pairs.iter()
                .map(|(id, _)| *id)
                .collect();
            
            let new_pair_id = Uuid::new_v4();
            
            self.stats.distillations_performed += 1;
            
            RecyclingResult {
                recycled_pairs: recycled_ids,
                extracted_ergotropy,
                dissipated_entropy: total_entropy,
                generated_pairs: vec![new_pair_id],
                strategy: RecyclingStrategy::Distillation,
                processing_time_ns: 0,
                success: true,
            }
        } else {
            // Insufficient pairs; perform simple extraction
            self.extract_ergotropy(pair_id, state)
        }
    }
    
    /// Simple discard
    fn discard_pair(
        &self,
        pair_id: Uuid,
        state: &EntangledPairState,
    ) -> RecyclingResult {
        RecyclingResult {
            recycled_pairs: vec![pair_id],
            extracted_ergotropy: 0.0,
            dissipated_entropy: state.entanglement_entropy,
            generated_pairs: Vec::new(),
            strategy: RecyclingStrategy::Discard,
            processing_time_ns: 0,
            success: true,
        }
    }
    
    // ========================================================================
    // ESTATÍSTICAS
    // ========================================================================
    
    fn update_stats(&mut self, result: &RecyclingResult) {
        if result.success {
            self.stats.total_recycled += result.recycled_pairs.len() as u64;
            self.stats.total_ergotropy_extracted += result.extracted_ergotropy;
            self.stats.total_entropy_dissipated += result.dissipated_entropy;
            self.stats.pairs_generated += result.generated_pairs.len() as u64;
            
            // Update success rate
            let total_operations = self.stats.total_recycled as f64;
            self.stats.success_rate = (self.stats.pairs_generated as f64) / total_operations;
        }
    }
    
    pub fn get_statistics(&self) -> &RecyclingStatistics {
        &self.stats
    }
    
    pub fn reset_statistics(&mut self) {
        self.stats = RecyclingStatistics::default();
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    fn current_time_ns() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
    
    /// Returns queue size
    pub fn queue_size(&self) -> usize {
        self.recycling_queue.read().unwrap().len()
    }
    
    /// Clears queue
    pub fn clear_queue(&mut self) {
        self.recycling_queue.write().unwrap().clear();
    }
}

// ============================================================================
// LIFECYCLE MANAGER
// ============================================================================

/// Complete TUCU lifecycle manager
pub struct EntanglementLifecycleManager {
    recycler: EntanglementRecycler,
    
    /// Phase counters
    phase_counts: HashMap<ThermodynamicPhase, usize>,
    
    /// Transition history
    transition_history: VecDeque<PhaseTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub pair_id: Uuid,
    pub from_phase: ThermodynamicPhase,
    pub to_phase: ThermodynamicPhase,
    pub timestamp: u64,
    pub fidelity: f64,
    pub ergotropy: f64,
}

impl EntanglementLifecycleManager {
    pub fn new(recycler: EntanglementRecycler) -> Self {
        Self {
            recycler,
            phase_counts: HashMap::new(),
            transition_history: VecDeque::new(),
        }
    }
    
    /// Updates phase counters
    pub fn update_phase_counts(
        &mut self,
        pool_pairs: &HashMap<Uuid, EntangledPairState>,
    ) {
        self.phase_counts.clear();
        
        for state in pool_pairs.values() {
            let phase = ThermodynamicPhaseClassifier::classify_phase(state);
            *self.phase_counts.entry(phase).or_insert(0) += 1;
        }
    }
    
    /// Records phase transition
    pub fn record_transition(
        &mut self,
        pair_id: Uuid,
        from_phase: ThermodynamicPhase,
        to_phase: ThermodynamicPhase,
        fidelity: f64,
        ergotropy: f64,
    ) {
        let transition = PhaseTransition {
            pair_id,
            from_phase,
            to_phase,
            timestamp: Self::current_timestamp(),
            fidelity,
            ergotropy,
        };
        
        self.transition_history.push_back(transition);
        
        // Keep limited history
        if self.transition_history.len() > 1000 {
            self.transition_history.pop_front();
        }
    }
    
    /// Returns phase distribution
    pub fn phase_distribution(&self) -> HashMap<ThermodynamicPhase, f64> {
        let total: usize = self.phase_counts.values().sum();
        
        if total == 0 {
            return HashMap::new();
        }
        
        self.phase_counts
            .iter()
            .map(|(phase, count)| {
                (*phase, *count as f64 / total as f64)
            })
            .collect()
    }
    
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::{Complex, DMatrix};
    
    fn create_test_state(fidelity: f64, ergotropy: f64) -> EntangledPairState {
        EntangledPairState {
            density_matrix: vec![],
            fidelity,
            concurrence: 0.8,
            entanglement_entropy: 0.5,
            ergotropy,
            negativity: 0.4,
            created_at: 0,
            reuse_count: 0,
        }
    }
    
    #[test]
    fn test_recycling_policy_conservative() {
        let policy = RecyclingPolicy::Conservative;
        
        // Pair em Radiation deve ser reciclado
        let state_radiation = create_test_state(0.4, 0.1);
        assert!(policy.should_recycle(&state_radiation, 0.5));
        
        // Pair in Degradation should not be recycled
        let state_degradation = create_test_state(0.7, 0.3);
        assert!(!policy.should_recycle(&state_degradation, 0.5));
    }
    
    #[test]
    fn test_recycling_policy_aggressive() {
        let policy = RecyclingPolicy::Aggressive { threshold: 80 };
        
        // Fidelity < 80% should be recycled
        let state_low = create_test_state(0.75, 0.3);
        assert!(policy.should_recycle(&state_low, 0.5));
        
        // Fidelity >= 80% should not be recycled
        let state_high = create_test_state(0.85, 0.5);
        assert!(!policy.should_recycle(&state_high, 0.5));
    }
    
    #[test]
    fn test_enqueue_recycling() {
        let mut recycler = EntanglementRecycler::new(
            RecyclingPolicy::Conservative,
            RecyclingStrategy::ErgotropyExtraction,
        );
        
        let pair_id = Uuid::new_v4();
        
        assert!(recycler.enqueue_for_recycling(pair_id).is_ok());
        assert_eq!(recycler.queue_size(), 1);
    }
    
    #[test]
    fn test_ergotropy_extraction() {
        let recycler = EntanglementRecycler::new(
            RecyclingPolicy::Conservative,
            RecyclingStrategy::ErgotropyExtraction,
        );
        
        let pair_id = Uuid::new_v4();
        let state = create_test_state(0.4, 0.5);
        
        let result = recycler.extract_ergotropy(pair_id, &state);
        
        assert!(result.success);
        assert_eq!(result.extracted_ergotropy, 0.5);
        assert_eq!(result.recycled_pairs.len(), 1);
    }
    
    #[test]
    fn test_lifecycle_manager() {
        let recycler = EntanglementRecycler::new(
            RecyclingPolicy::Balanced,
            RecyclingStrategy::Purification,
        );
        
        let mut manager = EntanglementLifecycleManager::new(recycler);
        
        let mut pool = HashMap::new();
        pool.insert(Uuid::new_v4(), create_test_state(0.95, 0.8));  // Generation
        pool.insert(Uuid::new_v4(), create_test_state(0.7, 0.3));   // Degradation
        pool.insert(Uuid::new_v4(), create_test_state(0.4, 0.1));   // Radiation
        
        manager.update_phase_counts(&pool);
        
        let distribution = manager.phase_distribution();
        assert_eq!(distribution.len(), 3);
    }
}
