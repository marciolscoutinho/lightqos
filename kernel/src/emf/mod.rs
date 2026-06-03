// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// mod.rs — EMF module — Entangled Memory Fabric public interface
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 04-08-2025
// All rights reserved.
// ---------------------------------------------------------------------------

pub mod entanglement_pool;
pub mod metrics;
pub mod pser_routing;
pub mod recycler;

// Re-export main structures
pub use metrics::{
    EMFPoolMetrics, EMFPoolMetricsAggregator, EntangledPairState, EntanglementMetricsCalculator,
    ThermodynamicPhase, ThermodynamicPhaseClassifier,
};

pub use pser_routing::{
    CalculatedRoute, NetworkLink, NetworkNode, NetworkTopology, PSERRouter, RoutingMetric,
    RoutingRequest, RoutingStatistics,
};

pub use recycler::{
    EntanglementLifecycleManager, EntanglementRecycler, PhaseTransition, RecyclerConfig,
    RecyclingPolicy, RecyclingResult, RecyclingStatistics, RecyclingStrategy,
};

pub use entanglement_pool::{EntanglementPool, PoolConfig, PoolStatistics};
