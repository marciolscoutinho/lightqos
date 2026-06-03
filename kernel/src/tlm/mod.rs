// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — TLM module — Temporal Layer Manager public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 30-06-2024
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod contract;
pub mod harmonic_scheduler;
pub mod process_tensor;
pub mod snapshot;

// Re-export main structures

// Contracts
pub use contract::{
    BandwidthContract, CoherenceContract, ContractManager, ContractSeverity, ContractStatistics,
    ContractType, FidelityContract, TemporalContract, TemporalVariant, ValidationResult,
    ViolationAction, ViolationType,
};

// Process Tensors
pub use process_tensor::{
    NonMarkovianityMetrics, ProcessTensor, ProcessTensorBuilder, ProcessTensorCache,
};

// Snapshots
pub use snapshot::{
    ExpirationPolicy, GateType, Operation, QuantumSnapshot, QuantumTransaction, RollbackResult,
    SnapshotDiff, SnapshotManager, SnapshotStatistics,
};

// Scheduler
pub use harmonic_scheduler::{
    ElectromagneticOctave, HarmonicScheduler, QuantumTask, SchedulerConfig, SchedulerStatistics,
    TaskExecutionResult,
};
