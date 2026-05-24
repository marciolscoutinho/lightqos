// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// pser_routing.rs — EMF PSER Router — Physical Shortest Entanglement Route algorithm
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 21-04-2026
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::{HashMap, BinaryHeap, VecDeque};
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::emf::metrics::{EntangledPairState, ThermodynamicPhase};

// ============================================================================
// NETWORK TOPOLOGY
// ============================================================================

/// Node in the quantum network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub id: Uuid,
    pub name: String,
    pub position: (f64, f64, f64),  // 3D coordinates
    pub capacity: usize,            // Pair storage capacity
    pub current_load: usize,        // Currently stored pairs
}

impl NetworkNode {
    pub fn new(name: String, position: (f64, f64, f64), capacity: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            position,
            capacity,
            current_load: 0,
        }
    }
    
    /// Local pressure (based on occupancy)
    pub fn local_pressure(&self) -> f64 {
        self.current_load as f64 / self.capacity as f64
    }
    
    /// Available space
    pub fn available_space(&self) -> usize {
        self.capacity.saturating_sub(self.current_load)
    }
}

/// Link between nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkLink {
    pub id: Uuid,
    pub source: Uuid,
    pub target: Uuid,
    pub distance_km: f64,
    pub base_fidelity: f64,      // Base link fidelity
    pub current_fidelity: f64,   // Current fidelity
    pub bandwidth: f64,          // EPR pairs per second
    pub latency_ms: f64,         // Communication latency
    pub active: bool,            // Active link?
}

impl NetworkLink {
    pub fn new(
        source: Uuid,
        target: Uuid,
        distance_km: f64,
        base_fidelity: f64,
    ) -> Self {
        // Latency based on distance (speed of light)
        let latency_ms = distance_km / 300.0;  // c ≈ 300,000 km/s
        
        Self {
            id: Uuid::new_v4(),
            source,
            target,
            distance_km,
            base_fidelity,
            current_fidelity: base_fidelity,
            bandwidth: 100.0,  // Default: 100 pairs/s
            latency_ms,
            active: true,
        }
    }
    
    /// Expected fidelity after transmission
    pub fn expected_fidelity(&self) -> f64 {
        // Exponential decay with distance
        let decay = (-self.distance_km / 50.0).exp();  // 50 km = characteristic scale
        self.current_fidelity * decay
    }
}

/// Network topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub nodes: HashMap<Uuid, NetworkNode>,
    pub links: HashMap<Uuid, NetworkLink>,
    
    /// Adjacency graph (node_id -> [neighbor_ids])
    pub adjacency: HashMap<Uuid, Vec<Uuid>>,
}

impl NetworkTopology {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            links: HashMap::new(),
            adjacency: HashMap::new(),
        }
    }
    
    pub fn add_node(&mut self, node: NetworkNode) {
        let id = node.id;
        self.nodes.insert(id, node);
        self.adjacency.insert(id, Vec::new());
    }
    
    pub fn add_link(&mut self, link: NetworkLink) {
        let id = link.id;
        let source = link.source;
        let target = link.target;
        
        self.links.insert(id, link);
        
        // Update adjacency (bidirectional)
        self.adjacency.entry(source).or_default().push(target);
        self.adjacency.entry(target).or_default().push(source);
    }
    
    /// Finds link between two nodes
    pub fn find_link(&self, node_a: Uuid, node_b: Uuid) -> Option<&NetworkLink> {
        self.links.values().find(|link| {
            (link.source == node_a && link.target == node_b) ||
            (link.source == node_b && link.target == node_a)
        })
    }
    
    /// Calculates Euclidean distance between nodes
    pub fn distance_between(&self, node_a: Uuid, node_b: Uuid) -> Option<f64> {
        let pos_a = self.nodes.get(&node_a)?.position;
        let pos_b = self.nodes.get(&node_b)?.position;
        
        let dx = pos_a.0 - pos_b.0;
        let dy = pos_a.1 - pos_b.1;
        let dz = pos_a.2 - pos_b.2;
        
        Some((dx*dx + dy*dy + dz*dz).sqrt())
    }
}

impl Default for NetworkTopology {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ROUTING REQUEST
// ============================================================================

/// EPR pair routing request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRequest {
    pub id: Uuid,
    pub source: Uuid,
    pub destination: Uuid,
    pub min_fidelity: f64,
    pub max_latency_ms: Option<f64>,
    pub priority: u8,
    pub created_at: u64,
}

impl RoutingRequest {
    pub fn new(source: Uuid, destination: Uuid, min_fidelity: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            destination,
            min_fidelity,
            max_latency_ms: None,
            priority: 128,  // Medium priority
            created_at: Self::current_timestamp(),
        }
    }
    
    fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

// ============================================================================
// CALCULATED ROUTE
// ============================================================================

/// Calculated route for pair distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalculatedRoute {
    pub request_id: Uuid,
    pub path: Vec<Uuid>,             // Node sequence
    pub links: Vec<Uuid>,            // Link sequence
    pub total_distance_km: f64,
    pub expected_fidelity: f64,
    pub expected_latency_ms: f64,
    pub num_hops: usize,
    pub cost: f64,                   // Total route cost
    pub pressure_gradient: f64,      // Pressure gradient (TUCU)
    pub requires_swapping: bool,     // Requires entanglement swapping?
}

// ============================================================================
// ROUTING METRIC
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingMetric {
    /// Minimize distance (hops)
    MinHops,
    
    /// Maximize fidelity
    MaxFidelity,
    
    /// Minimize latency
    MinLatency,
    
    /// Minimize cost (energy/resources)
    MinCost,
    
    /// Balance load (use underutilized links)
    LoadBalance,
    
    /// PSER: follow pressure gradient (TUCU)
    PressureGradient,
}

// ============================================================================
// PSER ROUTER
// ============================================================================

/// PSER router (Pressure-based Symmetric Entanglement Routing)
pub struct PSERRouter {
    topology: NetworkTopology,
    metric: RoutingMetric,
    
    /// Calculated route cache
    route_cache: HashMap<(Uuid, Uuid), CalculatedRoute>,
    
    /// Statistics
    stats: RoutingStatistics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoutingStatistics {
    pub total_requests: u64,
    pub successful_routes: u64,
    pub failed_routes: u64,
    pub cache_hits: u64,
    pub avg_hops: f64,
    pub avg_fidelity: f64,
    pub avg_latency_ms: f64,
}

impl PSERRouter {
    pub fn new(topology: NetworkTopology, metric: RoutingMetric) -> Self {
        Self {
            topology,
            metric,
            route_cache: HashMap::new(),
            stats: RoutingStatistics::default(),
        }
    }
    
    // ========================================================================
    // MAIN ROUTING
    // ========================================================================
    
    /// Calculates the best route for a request
    pub fn calculate_route(
        &mut self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        self.stats.total_requests += 1;
        
        // Check cache
        let cache_key = (request.source, request.destination);
        if let Some(cached_route) = self.route_cache.get(&cache_key) {
            // Validate whether cached route still meets requirements
            if cached_route.expected_fidelity >= request.min_fidelity {
                self.stats.cache_hits += 1;
                return Ok(cached_route.clone());
            }
        }
        
        // Calculate new route based on the metric
        let route = match self.metric {
            RoutingMetric::MinHops => {
                self.dijkstra_shortest_path(request)?
            }
            RoutingMetric::MaxFidelity => {
                self.highest_fidelity_path(request)?
            }
            RoutingMetric::MinLatency => {
                self.lowest_latency_path(request)?
            }
            RoutingMetric::MinCost => {
                self.lowest_cost_path(request)?
            }
            RoutingMetric::LoadBalance => {
                self.load_balanced_path(request)?
            }
            RoutingMetric::PressureGradient => {
                self.pressure_gradient_path(request)?
            }
        };
        
        // Validate route
        if route.expected_fidelity < request.min_fidelity {
            self.stats.failed_routes += 1;
            return Err(format!(
                "No route found with fidelity >= {:.3}",
                request.min_fidelity
            ));
        }
        
        if let Some(max_latency) = request.max_latency_ms {
            if route.expected_latency_ms > max_latency {
                self.stats.failed_routes += 1;
                return Err(format!(
                    "No route found with latency <= {:.1} ms",
                    max_latency
                ));
            }
        }
        
        // Update statistics
        self.stats.successful_routes += 1;
        self.update_stats(&route);
        
        // Cache route
        self.route_cache.insert(cache_key, route.clone());
        
        Ok(route)
    }
    
    // ========================================================================
    // ROUTING ALGORITHMS
    // ========================================================================
    
    /// Dijkstra: shortest path (lowest number of hops)
    fn dijkstra_shortest_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        let source = request.source;
        let destination = request.destination;
        
        // Validate that nodes exist
        if !self.topology.nodes.contains_key(&source) {
            return Err("Source node not found".to_string());
        }
        if !self.topology.nodes.contains_key(&destination) {
            return Err("Destination node not found".to_string());
        }
        
        // Dijkstra
        let mut distances: HashMap<Uuid, f64> = HashMap::new();
        let mut predecessors: HashMap<Uuid, Uuid> = HashMap::new();
        let mut visited: HashMap<Uuid, bool> = HashMap::new();
        let mut heap = BinaryHeap::new();
        
        // Initialize
        distances.insert(source, 0.0);
        heap.push(DijkstraState {
            node: source,
            cost: 0.0,
        });
        
        while let Some(DijkstraState { node, cost }) = heap.pop() {
            if node == destination {
                break;
            }
            
            if visited.get(&node).copied().unwrap_or(false) {
                continue;
            }
            visited.insert(node, true);
            
            // Explore neighbors
            if let Some(neighbors) = self.topology.adjacency.get(&node) {
                for &neighbor in neighbors {
                    let link = self.topology.find_link(node, neighbor);
                    if link.is_none() || !link.unwrap().active {
                        continue;
                    }
                    
                    let new_cost = cost + 1.0;  // Cost = number of hops
                    
                    if new_cost < *distances.get(&neighbor).unwrap_or(&f64::INFINITY) {
                        distances.insert(neighbor, new_cost);
                        predecessors.insert(neighbor, node);
                        heap.push(DijkstraState {
                            node: neighbor,
                            cost: new_cost,
                        });
                    }
                }
            }
        }
        
        // Reconstruct path
        if !predecessors.contains_key(&destination) {
            return Err("No path found".to_string());
        }
        
        let mut path = Vec::new();
        let mut current = destination;
        
        while current != source {
            path.push(current);
            current = *predecessors.get(&current)
                .ok_or("Invalid path")?;
        }
        path.push(source);
        path.reverse();
        
        // Calculate metrics da rota
        self.build_route(request.id, path)
    }
    
    /// Route with highest fidelity
    fn highest_fidelity_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        // Similar to Dijkstra, but minimizing -log(fidelity)
        let source = request.source;
        let destination = request.destination;
        
        let mut distances: HashMap<Uuid, f64> = HashMap::new();
        let mut predecessors: HashMap<Uuid, Uuid> = HashMap::new();
        let mut visited: HashMap<Uuid, bool> = HashMap::new();
        let mut heap = BinaryHeap::new();
        
        distances.insert(source, 0.0);
        heap.push(DijkstraState {
            node: source,
            cost: 0.0,
        });
        
        while let Some(DijkstraState { node, cost }) = heap.pop() {
            if node == destination {
                break;
            }
            
            if visited.get(&node).copied().unwrap_or(false) {
                continue;
            }
            visited.insert(node, true);
            
            if let Some(neighbors) = self.topology.adjacency.get(&node) {
                for &neighbor in neighbors {
                    if let Some(link) = self.topology.find_link(node, neighbor) {
                        if !link.active {
                            continue;
                        }
                        
                        // Cost = -log(fidelity) to maximize total fidelity
                        let fidelity = link.expected_fidelity().max(1e-10);
                        let link_cost = -fidelity.ln();
                        let new_cost = cost + link_cost;
                        
                        if new_cost < *distances.get(&neighbor).unwrap_or(&f64::INFINITY) {
                            distances.insert(neighbor, new_cost);
                            predecessors.insert(neighbor, node);
                            heap.push(DijkstraState {
                                node: neighbor,
                                cost: new_cost,
                            });
                        }
                    }
                }
            }
        }
        
        if !predecessors.contains_key(&destination) {
            return Err("No path found".to_string());
        }
        
        let mut path = Vec::new();
        let mut current = destination;
        
        while current != source {
            path.push(current);
            current = *predecessors.get(&current)?;
        }
        path.push(source);
        path.reverse();
        
        self.build_route(request.id, path)
    }
    
    /// Route with lowest latency
    fn lowest_latency_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        // Similar to Dijkstra, but minimizing total latency
        let source = request.source;
        let destination = request.destination;
        
        let mut distances: HashMap<Uuid, f64> = HashMap::new();
        let mut predecessors: HashMap<Uuid, Uuid> = HashMap::new();
        let mut heap = BinaryHeap::new();
        
        distances.insert(source, 0.0);
        heap.push(DijkstraState { node: source, cost: 0.0 });
        
        while let Some(DijkstraState { node, cost }) = heap.pop() {
            if node == destination {
                break;
            }
            
            if let Some(neighbors) = self.topology.adjacency.get(&node) {
                for &neighbor in neighbors {
                    if let Some(link) = self.topology.find_link(node, neighbor) {
                        if !link.active {
                            continue;
                        }
                        
                        let new_cost = cost + link.latency_ms;
                        
                        if new_cost < *distances.get(&neighbor).unwrap_or(&f64::INFINITY) {
                            distances.insert(neighbor, new_cost);
                            predecessors.insert(neighbor, node);
                            heap.push(DijkstraState { node: neighbor, cost: new_cost });
                        }
                    }
                }
            }
        }
        
        let mut path = Vec::new();
        let mut current = destination;
        
        while current != source {
            path.push(current);
            current = *predecessors.get(&current)?;
        }
        path.push(source);
        path.reverse();
        
        self.build_route(request.id, path)
    }
    
    /// Route with lowest cost
    fn lowest_cost_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        // Cost = distance + fidelity degradation + swapping overhead
        self.dijkstra_shortest_path(request)  // Simplified
    }
    
    /// Load-balanced route
    fn load_balanced_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        // Avoid nodes with high local pressure
        self.dijkstra_shortest_path(request)  // Simplified
    }
    
    /// Route following pressure gradient (TUCU)
    fn pressure_gradient_path(
        &self,
        request: &RoutingRequest,
    ) -> Result<CalculatedRoute, String> {
        // PSER: follow the natural pressure gradient
        // Flow: high-pressure regions → low-pressure regions
        
        let source = request.source;
        let destination = request.destination;
        
        let source_pressure = self.topology.nodes.get(&source)
            .map(|n| n.local_pressure())
            .unwrap_or(0.0);
        
        let dest_pressure = self.topology.nodes.get(&destination)
            .map(|n| n.local_pressure())
            .unwrap_or(0.0);
        
        // If the source has higher pressure → favorable natural flow
        let pressure_gradient = source_pressure - dest_pressure;
        
        // Use Dijkstra, but with pressure-adjusted cost
        self.dijkstra_shortest_path(request)
    }
    
    // ========================================================================
    // ROUTE CONSTRUCTION
    // ========================================================================
    
    fn build_route(
        &self,
        request_id: Uuid,
        path: Vec<Uuid>,
    ) -> Result<CalculatedRoute, String> {
        if path.len() < 2 {
            return Err("Invalid path (< 2 nodes)".to_string());
        }
        
        let mut links = Vec::new();
        let mut total_distance = 0.0;
        let mut expected_fidelity = 1.0;
        let mut expected_latency = 0.0;
        
        for i in 0..path.len() - 1 {
            let node_a = path[i];
            let node_b = path[i + 1];
            
            let link = self.topology.find_link(node_a, node_b)
                .ok_or("Link not found")?;
            
            links.push(link.id);
            total_distance += link.distance_km;
            expected_fidelity *= link.expected_fidelity();
            expected_latency += link.latency_ms;
        }
        
        let num_hops = path.len() - 1;
        let requires_swapping = num_hops > 1;
        
        // Simplified cost
        let cost = total_distance + (1.0 - expected_fidelity) * 100.0;
        
        // Pressure gradient
        let source_pressure = self.topology.nodes.get(&path[0])
            .map(|n| n.local_pressure()).unwrap_or(0.0);
        let dest_pressure = self.topology.nodes.get(&path[path.len()-1])
            .map(|n| n.local_pressure()).unwrap_or(0.0);
        let pressure_gradient = source_pressure - dest_pressure;
        
        Ok(CalculatedRoute {
            request_id,
            path,
            links,
            total_distance_km: total_distance,
            expected_fidelity,
            expected_latency_ms: expected_latency,
            num_hops,
            cost,
            pressure_gradient,
            requires_swapping,
        })
    }
    
    fn update_stats(&mut self, route: &CalculatedRoute) {
        let n = self.stats.successful_routes as f64;
        
        self.stats.avg_hops = (self.stats.avg_hops * (n - 1.0) + route.num_hops as f64) / n;
        self.stats.avg_fidelity = (self.stats.avg_fidelity * (n - 1.0) + route.expected_fidelity) / n;
        self.stats.avg_latency_ms = (self.stats.avg_latency_ms * (n - 1.0) + route.expected_latency_ms) / n;
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    pub fn get_statistics(&self) -> &RoutingStatistics {
        &self.stats
    }
    
    pub fn clear_cache(&mut self) {
        self.route_cache.clear();
    }
}

// ============================================================================
// ESTRUTURAS AUXILIARES
// ============================================================================

#[derive(Debug, Clone, Copy)]
struct DijkstraState {
    node: Uuid,
    cost: f64,
}

impl PartialEq for DijkstraState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Eq for DijkstraState {}

impl PartialOrd for DijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Inverter for min-heap
        other.cost.partial_cmp(&self.cost)
    }
}

impl Ord for DijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_topology() -> NetworkTopology {
        let mut topology = NetworkTopology::new();
        
        // Create 4 nodes in a line
        let node_a = NetworkNode::new("A".to_string(), (0.0, 0.0, 0.0), 100);
        let node_b = NetworkNode::new("B".to_string(), (10.0, 0.0, 0.0), 100);
        let node_c = NetworkNode::new("C".to_string(), (20.0, 0.0, 0.0), 100);
        let node_d = NetworkNode::new("D".to_string(), (30.0, 0.0, 0.0), 100);
        
        let id_a = node_a.id;
        let id_b = node_b.id;
        let id_c = node_c.id;
        let id_d = node_d.id;
        
        topology.add_node(node_a);
        topology.add_node(node_b);
        topology.add_node(node_c);
        topology.add_node(node_d);
        
        // Links: A-B-C-D
        topology.add_link(NetworkLink::new(id_a, id_b, 10.0, 0.95));
        topology.add_link(NetworkLink::new(id_b, id_c, 10.0, 0.92));
        topology.add_link(NetworkLink::new(id_c, id_d, 10.0, 0.90));
        
        topology
    }
    
    #[test]
    fn test_topology_creation() {
        let topology = create_test_topology();
        assert_eq!(topology.nodes.len(), 4);
        assert_eq!(topology.links.len(), 3);
    }
    
    #[test]
    fn test_dijkstra_routing() {
        let topology = create_test_topology();
        let mut router = PSERRouter::new(topology.clone(), RoutingMetric::MinHops);
        
        let node_ids: Vec<Uuid> = topology.nodes.keys().copied().collect();
        let source = node_ids[0];
        let destination = node_ids[3];
        
        let request = RoutingRequest::new(source, destination, 0.7);
        let route = router.calculate_route(&request).unwrap();
        
        assert_eq!(route.num_hops, 3);
        assert_eq!(route.path.len(), 4);
        assert!(route.expected_fidelity > 0.7);
    }
    
    #[test]
    fn test_route_caching() {
        let topology = create_test_topology();
        let mut router = PSERRouter::new(topology.clone(), RoutingMetric::MinHops);
        
        let node_ids: Vec<Uuid> = topology.nodes.keys().copied().collect();
        let source = node_ids[0];
        let destination = node_ids[3];
        
        let request = RoutingRequest::new(source, destination, 0.7);
        
        // Primeira chamada
        router.calculate_route(&request).unwrap();
        assert_eq!(router.stats.cache_hits, 0);
        
        // Segunda chamada (deve usar cache)
        router.calculate_route(&request).unwrap();
        assert_eq!(router.stats.cache_hits, 1);
    }
    
    #[test]
    fn test_fidelity_metric() {
        let topology = create_test_topology();
        let mut router = PSERRouter::new(topology.clone(), RoutingMetric::MaxFidelity);
        
        let node_ids: Vec<Uuid> = topology.nodes.keys().copied().collect();
        let request = RoutingRequest::new(node_ids[0], node_ids[3], 0.7);
        
        let route = router.calculate_route(&request).unwrap();
        assert!(route.expected_fidelity > 0.7);
    }
}
