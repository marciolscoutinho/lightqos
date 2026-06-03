// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — HIO module — Holographic I/O (Shadow Tomography) public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 03-11-2022
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod observable_view;
pub mod shadow_copy;
pub mod statistical_guarantee;

// Re-export main structures

// Shadow Tomography
pub use shadow_copy::{
    CollectorConfig, CollectorStatistics, MeasurementSnapshot, PauliOperator, PauliString,
    QuantumShadow, SamplingStrategy, ShadowCollector, ShadowMetadata,
};

// Observable Views
pub use observable_view::{
    Observable, ObservableFactory, ObservableMetadata, ObservableType, ObservableView, ViewConfig,
    ViewManager, ViewStatistics,
};

// Statistical Guarantees
pub use statistical_guarantee::{
    ChernoffBound, ConfidenceInterval, ConvergenceAnalyzer, GuaranteedEstimator, HoeffdingBound,
    PACGuarantee,
};
