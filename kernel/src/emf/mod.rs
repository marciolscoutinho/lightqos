// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — EMF module — Entangled Memory Fabric public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 04-08-2025
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod metrics;
pub mod pser_routing;
pub mod recycler;
pub mod entanglement_pool;

// Re-export main structures
pub use metrics::{
    EntangledPairState,
    EntanglementMetricsCalculator,
    EMFPoolMetrics,
    EMFPoolMetricsAggregator,
    ThermodynamicPhase,
    ThermodynamicPhaseClassifier,
};

pub use pser_routing::{
    PSERRouter,
    NetworkTopology,
    NetworkNode,
    NetworkLink,
    RoutingRequest,
    CalculatedRoute,
    RoutingMetric,
    RoutingStatistics,
};

pub use recycler::{
    EntanglementRecycler,
    RecyclingPolicy,
    RecyclingStrategy,
    RecyclingResult,
    RecyclerConfig,
    RecyclingStatistics,
    EntanglementLifecycleManager,
    PhaseTransition,
};

pub use entanglement_pool::{
    EntanglementPool,
    PoolConfig,
    PoolStatistics,
};
