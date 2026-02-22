//! Fiber Bundles over Spacetime
//! Mathematical basis for EFAL (ether field as a fiber bundle)

use crate::math::geometric_algebra::GA3D;
use std::collections::HashMap;

/// Base space (laboratory spacetime)
pub struct SpaceTimeBase {
    /// Spatial dimensions
    pub spatial_dims: usize,
    
    /// Temporal dimension
    pub temporal_dim: usize,
    
    /// Discretization points (for computation)
    pub lattice_points: Vec<GA3D>,
}

impl SpaceTimeBase {
    pub fn new(spatial_dims: usize, temporal_dim: usize) -> Self {
        SpaceTimeBase {
            spatial_dims,
            temporal_dim,
            lattice_points: Vec::new(),
        }
    }
    
    /// Adds spatial discretization points
    pub fn add_lattice_point(&mut self, point: GA3D) {
        self.lattice_points.push(point);
    }
}

/// Fiber (local quantum state space)
pub struct Fiber {
    /// Dimension of the local Hilbert space
    pub hilbert_dim: usize,
    
    /// Basis of the state space
    pub basis: Vec<String>, // e.g., ["0", "1"] for a qubit
}

impl Fiber {
    /// Creates a fiber for a qubit
    pub fn qubit() -> Self {
        Fiber {
            hilbert_dim: 2,
            basis: vec!["0".to_string(), "1".to_string()],
        }
    }
    
    /// Creates a fiber for a qudit of dimension d
    pub fn qudit(d: usize) -> Self {
        let basis = (0..d).map(|i| i.to_string()).collect();
        Fiber {
            hilbert_dim: d,
            basis,
        }
    }
    
    /// Creates a high-dimensional fiber (37D as in the experiment)
    pub fn high_dimensional(d: usize) -> Self {
        Self::qudit(d)
    }
}

/// Full bundle E → M
pub struct FiberBundle {
    /// Base space M (spacetime)
    pub base: SpaceTimeBase,
    
    /// Typical fiber F
    pub typical_fiber: Fiber,
    
    /// Map of each base point → its fiber
    pub fibers: HashMap<String, Fiber>,
    
    /// Connections (gauge) between fibers
    pub connections: Vec<Connection>,
}

impl FiberBundle {
    pub fn new(base: SpaceTimeBase) -> Self {
        FiberBundle {
            base,
            typical_fiber: Fiber::qubit(),
            fibers: HashMap::new(),
            connections: Vec::new(),
        }
    }
    
    /// Attaches a fiber to a base point
    pub fn attach_fiber(&mut self, point_id: String, fiber: Fiber) {
        self.fibers.insert(point_id, fiber);
    }
    
    /// Adds a (gauge) connection between two points
    pub fn add_connection(&mut self, from: String, to: String, connection_type: ConnectionType) {
        let conn = Connection {
            from_point: from,
            to_point: to,
            connection_type,
            gauge_field: GaugeField::identity(),
        };
        
        self.connections.push(conn);
    }
    
    /// Global section (choosing a state in each fiber)
    pub fn create_section(&self, state_selector: impl Fn(&str) -> Vec<f64>) -> Section {
        let mut states = HashMap::new();
        
        for (point_id, fiber) in &self.fibers {
            let state = state_selector(point_id);
            if state.len() == fiber.hilbert_dim {
                states.insert(point_id.clone(), state);
            }
        }
        
        Section { states }
    }
}

/// Gauge connection (parallel transport between fibers)
pub struct Connection {
    pub from_point: String,
    pub to_point: String,
    pub connection_type: ConnectionType,
    pub gauge_field: GaugeField,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
    Optical,      // Optical channel
    Microwave,    // Microwave
    Phononic,     // Phonons
    Virtual,      // Virtual connection (via entanglement)
}

/// Gauge field (connection potential)
pub struct GaugeField {
    /// Transport matrix (simplified as a phase)
    pub phase: f64,
    pub amplitude: f64,
}

impl GaugeField {
    pub fn identity() -> Self {
        GaugeField {
            phase: 0.0,
            amplitude: 1.0,
        }
    }
    
    /// Applies transport to a state
    pub fn transport(&self, state: &[f64]) -> Vec<f64> {
        state.iter()
            .map(|&s| s * self.amplitude * self.phase.cos())
            .collect()
    }
}

/// Bundle section (global state)
pub struct Section {
    /// State at each base point
    pub states: HashMap<String, Vec<f64>>,
}

impl Section {
    /// Evolves the section along the connections
    pub fn evolve_along(&mut self, connections: &[Connection]) {
        for conn in connections {
            if let Some(state) = self.states.get(&conn.from_point) {
                let transported = conn.gauge_field.transport(state);
                self.states.insert(conn.to_point.clone(), transported);
            }
        }
    }
}
