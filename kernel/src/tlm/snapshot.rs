// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// snapshot.rs — TLM Snapshot — quantum context capture and rollback mechanism
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 26-05-2026
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::{HashMap, VecDeque};
use serde::{Serialize, Deserialize};
use nalgebra::{Complex, DMatrix, DVector};
use uuid::Uuid;

// ============================================================================
// STATE SNAPSHOT
// ============================================================================

/// Snapshot of a quantum state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSnapshot {
    /// Unique snapshot ID
    pub id: Uuid,
    
    /// Snapshot name/label
    pub label: String,
    
    /// Creation timestamp (nanoseconds)
    pub timestamp_ns: u64,
    
    /// Complete state vector
    pub state_vector: Vec<Complex<f64>>,
    
    /// Density matrix (if applicable)
    pub density_matrix: Option<Vec<Complex<f64>>>,
    
    /// Operations applied up to this point
    pub operation_history: Vec<Operation>,
    
    /// Active contracts at snapshot time
    pub active_contracts: Vec<Uuid>,
    
    /// Additional metadata
    pub metadata: SnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Estimated fidelity
    pub estimated_fidelity: f64,
    
    /// Number of qubits
    pub num_qubits: usize,
    
    /// Global phase
    pub global_phase: f64,
    
    /// Free energy (if available)
    pub free_energy: Option<f64>,
    
    /// Entropy
    pub entropy: Option<f64>,
}

/// Applied operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub gate_type: GateType,
    pub qubits: Vec<usize>,
    pub parameters: Vec<f64>,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateType {
    H,      // Hadamard
    X,      // Pauli-X
    Y,      // Pauli-Y
    Z,      // Pauli-Z
    CNOT,   // Controlled-NOT
    CZ,     // Controlled-Z
    RX,     // X rotation
    RY,     // Y rotation
    RZ,     // Z rotation
    T,      // T gate
    S,      // S gate
    Custom, // Custom gate
}

impl QuantumSnapshot {
    /// Creates a new snapshot
    pub fn new(
        label: String,
        state_vector: Vec<Complex<f64>>,
        num_qubits: usize,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            label,
            timestamp_ns: Self::current_timestamp_ns(),
            state_vector,
            density_matrix: None,
            operation_history: Vec::new(),
            active_contracts: Vec::new(),
            metadata: SnapshotMetadata {
                estimated_fidelity: 1.0,
                num_qubits,
                global_phase: 0.0,
                free_energy: None,
                entropy: None,
            },
        }
    }
    
    /// Adds operation to history
    pub fn add_operation(&mut self, operation: Operation) {
        self.operation_history.push(operation);
    }
    
    /// Calculates difference with another snapshot
    pub fn diff(&self, other: &QuantumSnapshot) -> SnapshotDiff {
        // Calculate difference between state vectors
        let mut state_distance = 0.0;
        for (s1, s2) in self.state_vector.iter().zip(&other.state_vector) {
            state_distance += (s1 - s2).norm_sqr();
        }
        state_distance = state_distance.sqrt();
        
        // Different operations
        let ops_self: Vec<Uuid> = self.operation_history.iter().map(|o| o.id).collect();
        let ops_other: Vec<Uuid> = other.operation_history.iter().map(|o| o.id).collect();
        
        let mut operations_added = Vec::new();
        let mut operations_removed = Vec::new();
        
        for op_id in &ops_other {
            if !ops_self.contains(op_id) {
                operations_added.push(*op_id);
            }
        }
        
        for op_id in &ops_self {
            if !ops_other.contains(op_id) {
                operations_removed.push(*op_id);
            }
        }
        
        SnapshotDiff {
            from_id: self.id,
            to_id: other.id,
            state_distance,
            fidelity_change: other.metadata.estimated_fidelity - self.metadata.estimated_fidelity,
            operations_added,
            operations_removed,
            time_diff_ns: other.timestamp_ns.saturating_sub(self.timestamp_ns),
        }
    }
    
    fn current_timestamp_ns() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

/// Difference between two snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDiff {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub state_distance: f64,
    pub fidelity_change: f64,
    pub operations_added: Vec<Uuid>,
    pub operations_removed: Vec<Uuid>,
    pub time_diff_ns: u64,
}

// ============================================================================
// SNAPSHOT MANAGER
// ============================================================================

/// Snapshot manager with rollback
pub struct SnapshotManager {
    /// Stored snapshots (ID → Snapshot)
    snapshots: HashMap<Uuid, QuantumSnapshot>,
    
    /// Ordered timeline (timestamp → ID)
    timeline: VecDeque<(u64, Uuid)>,
    
    /// Current active snapshot
    current_snapshot: Option<Uuid>,
    
    /// Stored snapshot limit
    max_snapshots: usize,
    
    /// Expiration policy
    expiration_policy: ExpirationPolicy,
    
    /// Statistics
    stats: SnapshotStatistics,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExpirationPolicy {
    /// FIFO: remove oldest
    FIFO,
    
    /// LRU: remove least recently used
    LRU,
    
    /// Based on importance (fidelity)
    ImportanceBased,
    
    /// Keep all (no expiration)
    KeepAll,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SnapshotStatistics {
    pub total_snapshots_created: u64,
    pub total_rollbacks: u64,
    pub total_rollback_distance_ns: u64,
    pub avg_snapshot_size_bytes: usize,
    pub successful_rollbacks: u64,
    pub failed_rollbacks: u64,
}

impl SnapshotManager {
    pub fn new(max_snapshots: usize, policy: ExpirationPolicy) -> Self {
        Self {
            snapshots: HashMap::new(),
            timeline: VecDeque::new(),
            current_snapshot: None,
            max_snapshots,
            expiration_policy: policy,
            stats: SnapshotStatistics::default(),
        }
    }
    
    // ========================================================================
    // CREATION AND MANAGEMENT
    // ========================================================================
    
    /// Creates a snapshot of the current state
    pub fn create_snapshot(
        &mut self,
        label: String,
        state_vector: Vec<Complex<f64>>,
        num_qubits: usize,
    ) -> Uuid {
        let snapshot = QuantumSnapshot::new(label, state_vector, num_qubits);
        let id = snapshot.id;
        let timestamp = snapshot.timestamp_ns;
        
        // Check limit
        if self.snapshots.len() >= self.max_snapshots {
            self.expire_oldest();
        }
        
        // Add to timeline
        self.timeline.push_back((timestamp, id));
        
        // Store
        self.snapshots.insert(id, snapshot);
        self.current_snapshot = Some(id);
        
        // Statistics
        self.stats.total_snapshots_created += 1;
        
        id
    }
    
    /// Removes oldest snapshot based on policy
    fn expire_oldest(&mut self) {
        match self.expiration_policy {
            ExpirationPolicy::FIFO => {
                if let Some((_, id)) = self.timeline.pop_front() {
                    self.snapshots.remove(&id);
                }
            }
            
            ExpirationPolicy::LRU => {
                // Implement LRU tracking if necessary
                if let Some((_, id)) = self.timeline.pop_front() {
                    self.snapshots.remove(&id);
                }
            }
            
            ExpirationPolicy::ImportanceBased => {
                // Remove snapshot with lowest fidelity
                if let Some(lowest_fidelity_id) = self.find_lowest_fidelity_snapshot() {
                    self.snapshots.remove(&lowest_fidelity_id);
                    self.timeline.retain(|(_, id)| *id != lowest_fidelity_id);
                }
            }
            
            ExpirationPolicy::KeepAll => {
                // Do not remove
            }
        }
    }
    
    fn find_lowest_fidelity_snapshot(&self) -> Option<Uuid> {
        self.snapshots
            .iter()
            .min_by(|(_, a), (_, b)| {
                a.metadata.estimated_fidelity
                    .partial_cmp(&b.metadata.estimated_fidelity)
                    .unwrap()
            })
            .map(|(id, _)| *id)
    }
    
    /// Gets snapshot by ID
    pub fn get_snapshot(&self, id: Uuid) -> Option<&QuantumSnapshot> {
        self.snapshots.get(&id)
    }
    
    /// Gets snapshot by label
    pub fn get_snapshot_by_label(&self, label: &str) -> Option<&QuantumSnapshot> {
        self.snapshots
            .values()
            .find(|s| s.label == label)
    }
    
    /// Lists all snapshots
    pub fn list_snapshots(&self) -> Vec<&QuantumSnapshot> {
        let mut snapshots: Vec<&QuantumSnapshot> = self.snapshots.values().collect();
        snapshots.sort_by_key(|s| s.timestamp_ns);
        snapshots
    }
    
    // ========================================================================
    // ROLLBACK
    // ========================================================================
    
    /// Executes rollback to a specific snapshot
    pub fn rollback_to(&mut self, target_id: Uuid) -> Result<RollbackResult, String> {
        // Check whether snapshot exists
        let target = self.snapshots.get(&target_id)
            .ok_or("Snapshot not found")?
            .clone();
        
        // Get current snapshot
        let current_id = self.current_snapshot
            .ok_or("No current snapshot")?;
        
        let current = self.snapshots.get(&current_id)
            .ok_or("Current snapshot not found")?
            .clone();
        
        // Calculate difference
        let diff = current.diff(&target);
        
        // Execute rollback
        self.current_snapshot = Some(target_id);
        
        // Statistics
        self.stats.total_rollbacks += 1;
        self.stats.total_rollback_distance_ns += diff.time_diff_ns;
        self.stats.successful_rollbacks += 1;
        
        Ok(RollbackResult {
            from_id: current_id,
            to_id: target_id,
            diff,
            success: true,
            restored_state: target.state_vector.clone(),
        })
    }
    
    /// Rollback to snapshot by label
    pub fn rollback_to_label(&mut self, label: &str) -> Result<RollbackResult, String> {
        let snapshot = self.get_snapshot_by_label(label)
            .ok_or_else(|| format!("Snapshot '{}' not found", label))?;
        
        let id = snapshot.id;
        self.rollback_to(id)
    }
    
    /// Rollback to previous snapshot
    pub fn rollback_previous(&mut self) -> Result<RollbackResult, String> {
        let current_id = self.current_snapshot
            .ok_or("No current snapshot")?;
        
        // Find previous snapshot in timeline
        let current_timestamp = self.snapshots.get(&current_id)
            .ok_or("Current snapshot not found")?
            .timestamp_ns;
        
        let previous = self.timeline
            .iter()
            .rev()
            .find(|(ts, id)| *ts < current_timestamp && *id != current_id)
            .map(|(_, id)| *id)
            .ok_or("No previous snapshot found")?;
        
        self.rollback_to(previous)
    }
    
    /// Automatic rollback in case of contract violation
    pub fn auto_rollback_on_violation(
        &mut self,
        contract_id: Uuid,
    ) -> Result<RollbackResult, String> {
        // Find the last snapshot before the violation
        // Simplification: use previous snapshot
        self.rollback_previous()
    }
    
    // ========================================================================
    // TEMPORAL NAVIGATION
    // ========================================================================
    
    /// Advance to next snapshot (forward)
    pub fn forward_next(&mut self) -> Result<Uuid, String> {
        let current_id = self.current_snapshot
            .ok_or("No current snapshot")?;
        
        let current_timestamp = self.snapshots.get(&current_id)
            .ok_or("Current snapshot not found")?
            .timestamp_ns;
        
        let next = self.timeline
            .iter()
            .find(|(ts, id)| *ts > current_timestamp && *id != current_id)
            .map(|(_, id)| *id)
            .ok_or("No later snapshot found")?;
        
        self.current_snapshot = Some(next);
        Ok(next)
    }
    
    /// Return to the latest snapshot
    pub fn forward_to_latest(&mut self) -> Result<Uuid, String> {
        let latest = self.timeline.back()
            .map(|(_, id)| *id)
            .ok_or("No snapshot available")?;
        
        self.current_snapshot = Some(latest);
        Ok(latest)
    }
    
    // ========================================================================
    // COMPARISON AND ANALYSIS
    // ========================================================================
    
    /// Compares two snapshots
    pub fn compare(
        &self,
        id1: Uuid,
        id2: Uuid,
    ) -> Result<SnapshotDiff, String> {
        let s1 = self.snapshots.get(&id1)
            .ok_or("Snapshot 1 not found")?;
        let s2 = self.snapshots.get(&id2)
            .ok_or("Snapshot 2 not found")?;
        
        Ok(s1.diff(s2))
    }
    
    /// Finds snapshot with highest fidelity
    pub fn find_best_fidelity(&self) -> Option<Uuid> {
        self.snapshots
            .iter()
            .max_by(|(_, a), (_, b)| {
                a.metadata.estimated_fidelity
                    .partial_cmp(&b.metadata.estimated_fidelity)
                    .unwrap()
            })
            .map(|(id, _)| *id)
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    pub fn get_statistics(&self) -> &SnapshotStatistics {
        &self.stats
    }
    
    pub fn count(&self) -> usize {
        self.snapshots.len()
    }
    
    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.timeline.clear();
        self.current_snapshot = None;
    }
    
    pub fn get_current_snapshot(&self) -> Option<&QuantumSnapshot> {
        self.current_snapshot.and_then(|id| self.snapshots.get(&id))
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new(100, ExpirationPolicy::FIFO)
    }
}

/// Result de rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackResult {
    pub from_id: Uuid,
    pub to_id: Uuid,
    pub diff: SnapshotDiff,
    pub success: bool,
    pub restored_state: Vec<Complex<f64>>,
}

// ============================================================================
// QUANTUM TRANSACTIONS
// ============================================================================

/// Quantum transaction with adapted ACID semantics
pub struct QuantumTransaction {
    pub id: Uuid,
    pub initial_snapshot: Uuid,
    pub operations: Vec<Operation>,
    pub committed: bool,
    pub rollback_on_failure: bool,
}

impl QuantumTransaction {
    pub fn begin(snapshot_manager: &SnapshotManager) -> Result<Self, String> {
        let current = snapshot_manager.get_current_snapshot()
            .ok_or("No current snapshot")?;
        
        Ok(Self {
            id: Uuid::new_v4(),
            initial_snapshot: current.id,
            operations: Vec::new(),
            committed: false,
            rollback_on_failure: true,
        })
    }
    
    pub fn add_operation(&mut self, operation: Operation) {
        self.operations.push(operation);
    }
    
    pub fn commit(&mut self) {
        self.committed = true;
    }
    
    pub fn rollback(&mut self, snapshot_manager: &mut SnapshotManager) -> Result<(), String> {
        if self.rollback_on_failure {
            snapshot_manager.rollback_to(self.initial_snapshot)?;
        }
        Ok(())
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_state(amplitude: f64) -> Vec<Complex<f64>> {
        vec![
            Complex::new(amplitude, 0.0),
            Complex::new((1.0 - amplitude * amplitude).sqrt(), 0.0),
        ]
    }
    
    #[test]
    fn test_snapshot_creation() {
        let state = create_test_state(0.8);
        let snapshot = QuantumSnapshot::new("test".to_string(), state, 1);
        
        assert_eq!(snapshot.label, "test");
        assert_eq!(snapshot.metadata.num_qubits, 1);
    }
    
    #[test]
    fn test_manager_creation() {
        let manager = SnapshotManager::new(10, ExpirationPolicy::FIFO);
        assert_eq!(manager.count(), 0);
    }
    
    #[test]
    fn test_create_and_retrieve() {
        let mut manager = SnapshotManager::new(10, ExpirationPolicy::FIFO);
        
        let state = create_test_state(0.7);
        let id = manager.create_snapshot("checkpoint1".to_string(), state, 1);
        
        assert_eq!(manager.count(), 1);
        
        let retrieved = manager.get_snapshot(id);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().label, "checkpoint1");
    }
    
    #[test]
    fn test_rollback() {
        let mut manager = SnapshotManager::new(10, ExpirationPolicy::FIFO);
        
        // Create two snapshots
        let state1 = create_test_state(0.9);
        let id1 = manager.create_snapshot("s1".to_string(), state1, 1);
        
        let state2 = create_test_state(0.5);
        let id2 = manager.create_snapshot("s2".to_string(), state2, 1);
        
        // Rollback to s1
        let result = manager.rollback_to(id1).unwrap();
        
        assert!(result.success);
        assert_eq!(result.to_id, id1);
        assert_eq!(manager.current_snapshot, Some(id1));
    }
    
    #[test]
    fn test_expiration_policy() {
        let mut manager = SnapshotManager::new(2, ExpirationPolicy::FIFO);
        
        // Create 3 snapshots (the first one should expire)
        manager.create_snapshot("s1".to_string(), create_test_state(0.9), 1);
        manager.create_snapshot("s2".to_string(), create_test_state(0.8), 1);
        manager.create_snapshot("s3".to_string(), create_test_state(0.7), 1);
        
        assert_eq!(manager.count(), 2);
        
        // s1 should have been removed
        assert!(manager.get_snapshot_by_label("s1").is_none());
        assert!(manager.get_snapshot_by_label("s2").is_some());
        assert!(manager.get_snapshot_by_label("s3").is_some());
    }
    
    #[test]
    fn test_snapshot_diff() {
        let state1 = create_test_state(0.9);
        let state2 = create_test_state(0.5);
        
        let s1 = QuantumSnapshot::new("s1".to_string(), state1, 1);
        let s2 = QuantumSnapshot::new("s2".to_string(), state2, 1);
        
        let diff = s1.diff(&s2);
        
        assert!(diff.state_distance > 0.0);
    }
    
    #[test]
    fn test_transaction() {
        let mut manager = SnapshotManager::new(10, ExpirationPolicy::FIFO);
        let state = create_test_state(0.9);
        manager.create_snapshot("initial".to_string(), state, 1);
        
        let mut tx = QuantumTransaction::begin(&manager).unwrap();
        
        let op = Operation {
            id: Uuid::new_v4(),
            gate_type: GateType::H,
            qubits: vec![0],
            parameters: vec![],
            timestamp_ns: 0,
        };
        
        tx.add_operation(op);
        tx.commit();
        
        assert!(tx.committed);
    }
}
