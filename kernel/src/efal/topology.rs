// -----------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// topology.rs — EFAL topology abstractions
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// All rights reserved.
// -----------------------------------------------------------------------------

//! EFAL topology abstractions.
//!
//! This module defines basic topology structures used by the Entanglement Fabric
//! Abstraction Layer. It is intentionally minimal for now and can be expanded
//! with routing, connectivity maps, link fidelity models, and hardware topology
//! constraints.

/// Represents a logical quantum node in an EFAL topology.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologyNode {
    pub id: String,
}

/// Represents a logical connection between two quantum nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologyLink {
    pub source: String,
    pub target: String,
    pub fidelity: f64,
}

/// Basic EFAL topology container.
#[derive(Debug, Clone, Default)]
pub struct Topology {
    pub nodes: Vec<TopologyNode>,
    pub links: Vec<TopologyLink>,
}

impl Topology {
    /// Creates an empty topology.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node to the topology.
    pub fn add_node(&mut self, id: impl Into<String>) {
        self.nodes.push(TopologyNode { id: id.into() });
    }

    /// Adds a link between two nodes.
    pub fn add_link(
        &mut self,
        source: impl Into<String>,
        target: impl Into<String>,
        fidelity: f64,
    ) {
        self.links.push(TopologyLink {
            source: source.into(),
            target: target.into(),
            fidelity,
        });
    }

    /// Returns the number of nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns the number of links.
    pub fn link_count(&self) -> usize {
        self.links.len()
    }
}
