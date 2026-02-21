//! PSER - Pressure-Sensitive Entanglement Routing
//! 
//! Routing based on TUCU’s Electro-Gravitational Unification:
//! F_grav-rad = G·(m₁·m₂)/r² · (1 + α·∇P)

use crate::efal::geometry::CubeGeometry;
use crate::efal::topology::DynamicTopology;
use crate::math::geometric_algebra::GA3D;

/// Optimal route through the ether field
pub struct PSERRoute {
    pub waypoints: Vec<String>, // IDs of intermediate defects
    pub total_cost: f64,
    pub pressure_profile: Vec<f64>,
}

/// Computes the lowest-energy-cost route using the pressure gradient
pub fn compute_optimal_route(
    source: &str,
    target: &str,
    geometry: &CubeGeometry,
    topology: &DynamicTopology,
) -> Result<PSERRoute, String> {
    // Modified Dijkstra algorithm with PSER weighting
    
    let graph = topology.as_graph();
    let mut distances = HashMap::new();
    let mut previous = HashMap::new();
    let mut unvisited = HashSet::new();
    
    for node in graph.nodes() {
        distances.insert(node.clone(), f64::INFINITY);
        unvisited.insert(node.clone());
    }
    
    distances.insert(source.to_string(), 0.0);
    
    while !unvisited.is_empty() {
        // Node with the smallest distance
        let current = unvisited.iter()
            .min_by(|a, b| {
                distances[*a].partial_cmp(&distances[*b]).unwrap()
            })
            .unwrap()
            .clone();
        
        if current == target {
            break;
        }
        
        unvisited.remove(&current);
        
        for neighbor in graph.neighbors(&current) {
            let edge_cost = compute_pser_cost(
                &current,
                &neighbor,
                geometry,
                topology,
            );
            
            let alt = distances[&current] + edge_cost;
            
            if alt < distances[&neighbor] {
                distances.insert(neighbor.clone(), alt);
                previous.insert(neighbor.clone(), current.clone());
            }
        }
    }
    
    // Reconstructs the path
    let mut waypoints = Vec::new();
    let mut current = target.to_string();
    
    while current != source {
        waypoints.push(current.clone());
        current = previous.get(&current)
            .ok_or("No route found")?
            .clone();
    }
    
    waypoints.push(source.to_string());
    waypoints.reverse();
    
    Ok(PSERRoute {
        total_cost: distances[target],
        pressure_profile: compute_pressure_profile(&waypoints, geometry),
        waypoints,
    })
}

/// PSER cost between two adjacent nodes
fn compute_pser_cost(
    node_a: &str,
    node_b: &str,
    geometry: &CubeGeometry,
    topology: &DynamicTopology,
) -> f64 {
    // Defect positions in space
    let pos_a = topology.get_position(node_a).unwrap();
    let pos_b = topology.get_position(node_b).unwrap();
    
    // Physical distance
    let r = (pos_b - pos_a).magnitude();
    
    // Potential at the two points (simulated)
    let potential_a = estimate_potential(&pos_a, geometry);
    let potential_b = estimate_potential(&pos_b, geometry);
    
    // Potential difference
    let delta_p = (potential_b - potential_a).abs();
    
    // Reference potential (average)
    let p_0 = (potential_a + potential_b) / 2.0;
    
    // Sensitivity constant (tunable)
    const K_SENSITIVITY: f64 = 0.1;
    
    // PSER formula (based on TUCU):
    // F_attr/rep = G·(m₁·m₂)/r² · (1 + k·ΔP/P₀)
    // Cost is inversely proportional to the force (we want natural flow)
    
    let base_cost = r * r; // Proportional to r²
    let pressure_factor = 1.0 + K_SENSITIVITY * (delta_p / p_0.max(1e-6));
    
    base_cost / pressure_factor.max(0.1)
}

/// Estimates the local potential (simplified)
fn estimate_potential(position: &GA3D, geometry: &CubeGeometry) -> f64 {
    // Potential is higher near the inertial planes (inertia = 0=)
    // and decreases as you move away
    
    let dist_south = geometry.south_plane.distance_to(position);
    let dist_ns = geometry.north_south_plane.distance_to(position);
    let dist_ew = geometry.east_west_plane.distance_to(position);
    
    // Potential inversely proportional to the minimum distance
    let min_dist = dist_south.min(dist_ns).min(dist_ew);
    
    1.0 / (min_dist + 0.1) // Avoids division by zero
}

fn compute_pressure_profile(waypoints: &[String], geometry: &CubeGeometry) -> Vec<f64> {
    // Computes a pressure profile along the route
    waypoints.iter()
        .map(|wp| {
            // Simulation: pressure based on position
            0.0 // Placeholder
        })
        .collect()
}

use std::collections::{HashMap, HashSet};
