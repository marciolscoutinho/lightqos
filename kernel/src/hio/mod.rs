// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — HIO module — Holographic I/O (Shadow Tomography) public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 03-11-2022
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod shadow_copy;
pub mod observable_view;
pub mod statistical_guarantee;

// Re-export main structures

// Shadow Tomography
pub use shadow_copy::{
    QuantumShadow,
    ShadowCollector,
    MeasurementSnapshot,
    PauliString,
    PauliOperator,
    ShadowMetadata,
    SamplingStrategy,
    CollectorConfig,
    CollectorStatistics,
};

// Observable Views
pub use observable_view::{
    Observable,
    ObservableView,
    ViewManager,
    ObservableFactory,
    ObservableType,
    ObservableMetadata,
    ViewConfig,
    ViewStatistics,
};

// Statistical Guarantees
pub use statistical_guarantee::{
    PACGuarantee,
    HoeffdingBound,
    ChernoffBound,
    ConfidenceInterval,
    GuaranteedEstimator,
    ConvergenceAnalyzer,
};
