// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — TLM module — Temporal Layer Manager public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 30-06-2024
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod contract;
pub mod process_tensor;
pub mod snapshot;
pub mod harmonic_scheduler;

// Re-export main structures

// Contracts
pub use contract::{
    TemporalContract,
    FidelityContract,
    CoherenceContract,
    BandwidthContract,
    ContractManager,
    ContractType,
    ContractSeverity,
    TemporalVariant,
    ValidationResult,
    ViolationType,
    ViolationAction,
    ContractStatistics,
};

// Process Tensors
pub use process_tensor::{
    ProcessTensor,
    ProcessTensorBuilder,
    ProcessTensorCache,
    NonMarkovianityMetrics,
};

// Snapshots
pub use snapshot::{
    QuantumSnapshot,
    SnapshotManager,
    SnapshotDiff,
    RollbackResult,
    ExpirationPolicy,
    SnapshotStatistics,
    QuantumTransaction,
    Operation,
    GateType,
};

// Scheduler
pub use harmonic_scheduler::{
    HarmonicScheduler,
    QuantumTask,
    ElectromagneticOctave,
    TaskExecutionResult,
    SchedulerConfig,
    SchedulerStatistics,
};
