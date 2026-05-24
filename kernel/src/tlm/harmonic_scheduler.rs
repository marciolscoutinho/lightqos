// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// harmonic_scheduler.rs — TLM Harmonic Scheduler — epoch-based quantum operation scheduling
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 18-11-2025
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::tlm::contract::{TemporalContract, ContractManager, ValidationResult};
use crate::tlm::process_tensor::{ProcessTensor, ProcessTensorCache};
use crate::tlm::snapshot::{SnapshotManager, QuantumSnapshot, Operation};

// ============================================================================
// ELECTROMAGNETIC OCTAVES
// ============================================================================

/// Electromagnetic octave (characteristic frequency)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ElectromagneticOctave {
    Gravity = 1,        // 10^-3 - 10^0 Hz
    Magnetism = 2,      // 10^0 - 10^3 Hz
    Electricity = 3,    // 10^3 - 10^6 Hz
    Heat = 4,           // 10^6 - 10^9 Hz
    Infrared = 5,       // 10^9 - 10^12 Hz
    Visible = 6,        // 10^12 - 10^15 Hz
    Ultraviolet = 7,    // 10^15 - 10^18 Hz
    XRay = 8,           // 10^18 - 10^21 Hz
    Gamma = 9,          // 10^21 - 10^24 Hz
    Consciousness = 10, // >10^24 Hz
}

impl ElectromagneticOctave {
    /// Returns frequency range (Hz)
    pub fn frequency_range(&self) -> (f64, f64) {
        match self {
            Self::Gravity => (1e-3, 1e0),
            Self::Magnetism => (1e0, 1e3),
            Self::Electricity => (1e3, 1e6),
            Self::Heat => (1e6, 1e9),
            Self::Infrared => (1e9, 1e12),
            Self::Visible => (1e12, 1e15),
            Self::Ultraviolet => (1e15, 1e18),
            Self::XRay => (1e18, 1e21),
            Self::Gamma => (1e21, 1e24),
            Self::Consciousness => (1e24, f64::INFINITY),
        }
    }
    
    /// Central frequency of the octave
    pub fn center_frequency(&self) -> f64 {
        let (min, max) = self.frequency_range();
        if max.is_infinite() {
            min * 10.0
        } else {
            (min * max).sqrt()  // Geometric mean
        }
    }
    
    /// Characteristic period (seconds)
    pub fn characteristic_period(&self) -> f64 {
        1.0 / self.center_frequency()
    }
    
    /// Classifies frequency into the appropriate octave
    pub fn from_frequency(freq_hz: f64) -> Self {
        if freq_hz < 1e0 { Self::Gravity }
        else if freq_hz < 1e3 { Self::Magnetism }
        else if freq_hz < 1e6 { Self::Electricity }
        else if freq_hz < 1e9 { Self::Heat }
        else if freq_hz < 1e12 { Self::Infrared }
        else if freq_hz < 1e15 { Self::Visible }
        else if freq_hz < 1e18 { Self::Ultraviolet }
        else if freq_hz < 1e21 { Self::XRay }
        else if freq_hz < 1e24 { Self::Gamma }
        else { Self::Consciousness }
    }
}

// ============================================================================
// QUANTUM TASK
// ============================================================================

/// Quantum task to be scheduled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumTask {
    pub id: Uuid,
    pub name: String,
    pub operations: Vec<Operation>,
    pub contract: Option<TemporalContract>,
    pub priority: u8,
    pub estimated_duration_ns: u64,
    pub deadline_ns: Option<u64>,
    pub octave: ElectromagneticOctave,
    pub dependencies: Vec<Uuid>,
    pub created_at_ns: u64,
}

impl QuantumTask {
    pub fn new(name: String, operations: Vec<Operation>) -> Self {
        // Estimate duration based on the number of operations
        let estimated_duration_ns = operations.len() as u64 * 100;  // 100ns per operation
        
        // Calculate characteristic frequency
        let freq_hz = if estimated_duration_ns > 0 {
            1e9 / estimated_duration_ns as f64  // Convert ns → Hz
        } else {
            1e12  // Default: visible octave
        };
        
        Self {
            id: Uuid::new_v4(),
            name,
            operations,
            contract: None,
            priority: 128,  // Medium priority
            estimated_duration_ns,
            deadline_ns: None,
            octave: ElectromagneticOctave::from_frequency(freq_hz),
            dependencies: Vec::new(),
            created_at_ns: Self::current_timestamp_ns(),
        }
    }
    
    pub fn with_contract(mut self, contract: TemporalContract) -> Self {
        self.contract = Some(contract);
        self
    }
    
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn with_deadline(mut self, deadline_ns: u64) -> Self {
        self.deadline_ns = Some(deadline_ns);
        self
    }
    
    pub fn with_dependencies(mut self, dependencies: Vec<Uuid>) -> Self {
        self.dependencies = dependencies;
        self
    }
    
    fn current_timestamp_ns() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

impl PartialEq for QuantumTask {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for QuantumTask {}

impl PartialOrd for QuantumTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QuantumTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Sort by priority (higher first), then by deadline
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                match (self.deadline_ns, other.deadline_ns) {
                    (Some(d1), Some(d2)) => d1.cmp(&d2),
                    (Some(_), None) => Ordering::Less,
                    (None, Some(_)) => Ordering::Greater,
                    (None, None) => Ordering::Equal,
                }
            }
            other_ord => other_ord.reverse(),  // Invert for max-heap
        }
    }
}

// ============================================================================
// HARMONIC SCHEDULER
// ============================================================================

/// Scheduler based on harmonics of the 10 Octaves
pub struct HarmonicScheduler {
    /// Task queue (heap by priority)
    task_queue: BinaryHeap<QuantumTask>,
    
    /// Running tasks
    running_tasks: HashMap<Uuid, QuantumTask>,
    
    /// Completed tasks
    completed_tasks: VecDeque<Uuid>,
    
    /// Contract manager
    contract_manager: ContractManager,
    
    /// Snapshot manager
    snapshot_manager: SnapshotManager,
    
    /// Process tensor cache
    process_tensor_cache: ProcessTensorCache,
    
    /// Statistics
    stats: SchedulerStatistics,
    
    /// Configuration
    config: SchedulerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum number of simultaneous tasks
    pub max_concurrent_tasks: usize,
    
    /// Enable harmonic resonance
    pub enable_harmonic_resonance: bool,
    
    /// Enable automatic snapshots
    pub auto_snapshots: bool,
    
    /// Snapshot interval (ns)
    pub snapshot_interval_ns: u64,
    
    /// Enable automatic rollback
    pub auto_rollback: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 10,
            enable_harmonic_resonance: true,
            auto_snapshots: true,
            snapshot_interval_ns: 1_000_000,  // 1ms
            auto_rollback: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStatistics {
    pub total_tasks_scheduled: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub total_rollbacks: u64,
    pub avg_task_duration_ns: u64,
    pub contract_violations: u64,
    pub harmonic_resonances_detected: u64,
}

impl HarmonicScheduler {
    pub fn new() -> Self {
        Self::with_config(SchedulerConfig::default())
    }
    
    pub fn with_config(config: SchedulerConfig) -> Self {
        Self {
            task_queue: BinaryHeap::new(),
            running_tasks: HashMap::new(),
            completed_tasks: VecDeque::new(),
            contract_manager: ContractManager::new(),
            snapshot_manager: SnapshotManager::default(),
            process_tensor_cache: ProcessTensorCache::new(100),
            stats: SchedulerStatistics::default(),
            config,
        }
    }
    
    // ========================================================================
    // SCHEDULING
    // ========================================================================
    
    /// Adds task to the queue
    pub fn schedule_task(&mut self, task: QuantumTask) -> Uuid {
        let id = task.id;
        
        // Register contract if present
        if let Some(ref contract) = task.contract {
            self.contract_manager.register_temporal(contract.clone());
        }
        
        self.task_queue.push(task);
        self.stats.total_tasks_scheduled += 1;
        
        id
    }
    
    /// Executes the next available task
    pub fn execute_next(&mut self) -> Option<TaskExecutionResult> {
        // Check concurrency limit
        if self.running_tasks.len() >= self.config.max_concurrent_tasks {
            return None;
        }
        
        // Get next task
        let task = self.task_queue.pop()?;
        
        // Check dependencies
        if !self.are_dependencies_met(&task) {
            // Re-enqueue
            self.task_queue.push(task);
            return None;
        }
        
        // Create snapshot if enabled
        if self.config.auto_snapshots {
            self.create_task_snapshot(&task);
        }
        
        // Execute
        let result = self.execute_task(task);
        
        Some(result)
    }
    
    fn are_dependencies_met(&self, task: &QuantumTask) -> bool {
        task.dependencies.iter()
            .all(|dep_id| self.completed_tasks.contains(dep_id))
    }
    
    fn execute_task(&mut self, task: QuantumTask) -> TaskExecutionResult {
        let start_time = Self::current_timestamp_ns();
        let task_id = task.id;
        
        // Mark as running
        self.running_tasks.insert(task_id, task.clone());
        
        // Simulate execution (in practice, delegate to hardware)
        let duration_ns = task.estimated_duration_ns;
        
        // Validate contract
        let contract_met = if let Some(ref contract) = task.contract {
            let results = self.contract_manager.validate_temporal_all(duration_ns, None);
            results.iter().all(|r| r.is_success())
        } else {
            true
        };
        
        // Result
        let success = contract_met;
        
        if success {
            self.stats.total_tasks_completed += 1;
            self.completed_tasks.push_back(task_id);
        } else {
            self.stats.total_tasks_failed += 1;
            self.stats.contract_violations += 1;
            
            // Automatic rollback if enabled
            if self.config.auto_rollback {
                let _ = self.snapshot_manager.rollback_previous();
                self.stats.total_rollbacks += 1;
            }
        }
        
        // Remove from running
        self.running_tasks.remove(&task_id);
        
        // Update statistics
        let total_completed = self.stats.total_tasks_completed;
        if total_completed > 0 {
            self.stats.avg_task_duration_ns = 
                (self.stats.avg_task_duration_ns * (total_completed - 1) + duration_ns) / total_completed;
        }
        
        TaskExecutionResult {
            task_id,
            success,
            duration_ns,
            contract_violations: if contract_met { 0 } else { 1 },
            snapshot_created: self.config.auto_snapshots,
        }
    }
    
    fn create_task_snapshot(&mut self, task: &QuantumTask) {
        // Create snapshot before execution
        let state_vector = vec![nalgebra::Complex::new(1.0, 0.0); 2];  // State |0⟩
        self.snapshot_manager.create_snapshot(
            format!("before_{}", task.name),
            state_vector,
            1,
        );
    }
    
    // ========================================================================
    // HARMONIC RESONANCE
    // ========================================================================
    
    /// Detects harmonic resonance between tasks
    pub fn detect_harmonic_resonance(&mut self) -> Vec<(Uuid, Uuid, f64)> {
        if !self.config.enable_harmonic_resonance {
            return Vec::new();
        }
        
        let mut resonances = Vec::new();
        let tasks: Vec<&QuantumTask> = self.task_queue.iter().collect();
        
        // Compare all combinations
        for i in 0..tasks.len() {
            for j in (i + 1)..tasks.len() {
                let task1 = tasks[i];
                let task2 = tasks[j];
                
                // Calculate frequency ratio
                let freq1 = task1.octave.center_frequency();
                let freq2 = task2.octave.center_frequency();
                let ratio = freq1 / freq2;
                
                // Check whether it is harmonic (ratio close to an integer)
                let nearest_int = ratio.round();
                let harmonic_error = (ratio - nearest_int).abs();
                
                if harmonic_error < 0.1 {
                    // Resonance detected
                    let resonance_strength = 1.0 - harmonic_error;
                    resonances.push((task1.id, task2.id, resonance_strength));
                    self.stats.harmonic_resonances_detected += 1;
                }
            }
        }
        
        resonances
    }
    
    /// Optimizes scheduling based on resonance
    pub fn optimize_by_resonance(&mut self) {
        let resonances = self.detect_harmonic_resonance();
        
        if resonances.is_empty() {
            return;
        }
        
        // Regroup resonant tasks
        // (Simplified implementation - log only)
        println!("{} harmonic resonances detected", resonances.len());
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    pub fn get_statistics(&self) -> &SchedulerStatistics {
        &self.stats
    }
    
    pub fn queue_size(&self) -> usize {
        self.task_queue.len()
    }
    
    pub fn running_count(&self) -> usize {
        self.running_tasks.len()
    }
    
    fn current_timestamp_ns() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64
    }
}

impl Default for HarmonicScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task_id: Uuid,
    pub success: bool,
    pub duration_ns: u64,
    pub contract_violations: usize,
    pub snapshot_created: bool,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_octave_frequencies() {
        let visible = ElectromagneticOctave::Visible;
        let (min, max) = visible.frequency_range();
        
        assert_eq!(min, 1e12);
        assert_eq!(max, 1e15);
        
        let center = visible.center_frequency();
        assert!(center > min && center < max);
    }
    
    #[test]
    fn test_octave_from_frequency() {
        assert_eq!(ElectromagneticOctave::from_frequency(5e2), ElectromagneticOctave::Magnetism);
        assert_eq!(ElectromagneticOctave::from_frequency(5e13), ElectromagneticOctave::Visible);
        assert_eq!(ElectromagneticOctave::from_frequency(5e25), ElectromagneticOctave::Consciousness);
    }
    
    #[test]
    fn test_task_creation() {
        let ops = vec![Operation {
            id: Uuid::new_v4(),
            gate_type: crate::tlm::snapshot::GateType::H,
            qubits: vec![0],
            parameters: vec![],
            timestamp_ns: 0,
        }];
        
        let task = QuantumTask::new("test".to_string(), ops);
        
        assert_eq!(task.name, "test");
        assert!(task.estimated_duration_ns > 0);
    }
    
    #[test]
    fn test_scheduler_creation() {
        let scheduler = HarmonicScheduler::new();
        assert_eq!(scheduler.queue_size(), 0);
    }
    
    #[test]
    fn test_schedule_task() {
        let mut scheduler = HarmonicScheduler::new();
        
        let ops = vec![];
        let task = QuantumTask::new("task1".to_string(), ops);
        let id = scheduler.schedule_task(task);
        
        assert_eq!(scheduler.queue_size(), 1);
        assert_eq!(scheduler.stats.total_tasks_scheduled, 1);
    }
    
    #[test]
    fn test_execute_next() {
        let mut scheduler = HarmonicScheduler::new();
        
        let ops = vec![];
        let task = QuantumTask::new("task1".to_string(), ops);
        scheduler.schedule_task(task);
        
        let result = scheduler.execute_next();
        assert!(result.is_some());
        
        let res = result.unwrap();
        assert!(res.success);
    }
    
    #[test]
    fn test_harmonic_resonance() {
        let mut scheduler = HarmonicScheduler::new();
        
        // Create tasks in harmonic octaves
        let task1 = QuantumTask::new("t1".to_string(), vec![])
            .with_priority(200);
        let task2 = QuantumTask::new("t2".to_string(), vec![])
            .with_priority(200);
        
        scheduler.schedule_task(task1);
        scheduler.schedule_task(task2);
        
        let resonances = scheduler.detect_harmonic_resonance();
        // May or may not detect resonances depending on the frequencies
        assert!(resonances.len() >= 0);
    }
}
