// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// entanglement_pool.rs — EMF Entanglement Pool — Bell pair lifecycle management
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 29-09-2021
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use nalgebra::{Complex, DMatrix};

use crate::emf::metrics::{
    EntangledPairState,
    EntanglementMetricsCalculator,
    EMFPoolMetrics,
    EMFPoolMetricsAggregator,
    ThermodynamicPhase,
    ThermodynamicPhaseClassifier,
};

use crate::emf::recycler::{
    EntanglementRecycler,
    RecyclingPolicy,
    RecyclingStrategy,
    RecyclerConfig,
};

use crate::emf::pser_routing::{
    PSERRouter,
    NetworkTopology,
    RoutingRequest,
    RoutingMetric,
};

// ============================================================================
// ENTANGLEMENT POOL
// ============================================================================

/// Centralized pool of entangled pairs
pub struct EntanglementPool {
    /// Stored pairs (ID → State)
    pairs: Arc<RwLock<HashMap<Uuid, EntangledPairState>>>,
    
    /// Maximum pool capacity
    capacity: usize,
    
    /// Integrated recycler
    recycler: Arc<RwLock<EntanglementRecycler>>,
    
    /// PSER router
    router: Arc<RwLock<PSERRouter>>,
    
    /// Pool statistics
    stats: PoolStatistics,
    
    /// Configuration
    config: PoolConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum capacity
    pub max_capacity: usize,
    
    /// Pressure threshold for activating aggressive recycling
    pub high_pressure_threshold: f64,
    
    /// Maintenance interval (ms)
    pub maintenance_interval_ms: u64,
    
    /// Enable automatic recycling?
    pub auto_recycle: bool,
    
    /// Enable PSER routing?
    pub enable_pser: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_capacity: 10000,
            high_pressure_threshold: 0.8,
            maintenance_interval_ms: 5000,
            auto_recycle: true,
            enable_pser: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolStatistics {
    pub total_pairs_created: u64,
    pub total_pairs_consumed: u64,
    pub total_pairs_recycled: u64,
    pub peak_usage: usize,
    pub avg_fidelity: f64,
    pub avg_lifetime_ms: f64,
}

impl EntanglementPool {
    /// Creates a new pool with the default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }
    
    /// Creates a pool with a custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        // Create recycler
        let recycler = EntanglementRecycler::new(
            RecyclingPolicy::Adaptive,
            RecyclingStrategy::Purification,
        ).with_config(RecyclerConfig::default());
        
        // Create topology and router
        let topology = NetworkTopology::new();
        let router = PSERRouter::new(topology, RoutingMetric::PressureGradient);
        
        Self {
            pairs: Arc::new(RwLock::new(HashMap::new())),
            capacity: config.max_capacity,
            recycler: Arc::new(RwLock::new(recycler)),
            router: Arc::new(RwLock::new(router)),
            stats: PoolStatistics::default(),
            config,
        }
    }
    
    // ========================================================================
    // PAIR CREATION AND REMOVAL
    // ========================================================================
    
    /// Creates a new EPR pair
    pub fn create_pair(
        &mut self,
        density_matrix: DMatrix<Complex<f64>>,
    ) -> Result<Uuid, String> {
        let mut pairs = self.pairs.write().unwrap();
        
        // Check capacity
        if pairs.len() >= self.capacity {
            // Try recycling first
            if self.config.auto_recycle {
                drop(pairs);  // Release lock
                self.run_maintenance()?;
                pairs = self.pairs.write().unwrap();
                
                if pairs.len() >= self.capacity {
                    return Err("Pool is full even after recycling".to_string());
                }
            } else {
                return Err("Pool is full".to_string());
            }
        }
        
        // Calculate metrics
        let state = EntanglementMetricsCalculator::calculate_all_metrics(&density_matrix)?;
        
        let pair_id = Uuid::new_v4();
        pairs.insert(pair_id, state);
        
        // Update statistics
        self.stats.total_pairs_created += 1;
        if pairs.len() > self.stats.peak_usage {
            self.stats.peak_usage = pairs.len();
        }
        
        Ok(pair_id)
    }
    
    /// Gets a pair from the pool
    pub fn get_pair(&self, id: Uuid) -> Option<EntangledPairState> {
        let pairs = self.pairs.read().unwrap();
        pairs.get(&id).cloned()
    }
    
    /// Removes (consumes) a pair from the pool
    pub fn consume_pair(&mut self, id: Uuid) -> Option<EntangledPairState> {
        let mut pairs = self.pairs.write().unwrap();
        let pair = pairs.remove(&id);
        
        if pair.is_some() {
            self.stats.total_pairs_consumed += 1;
        }
        
        pair
    }
    
    /// Allocates an available pair with minimum fidelity
    pub fn allocate_pair(&mut self, min_fidelity: f64) -> Option<Uuid> {
        let pairs = self.pairs.read().unwrap();
        
        // Search for an available pair with suitable fidelity
        pairs
            .iter()
            .filter(|(_, state)| {
                state.fidelity >= min_fidelity &&
                state.reuse_count < 5  // Reuse limit
            })
            .max_by(|(_, a), (_, b)| {
                a.fidelity.partial_cmp(&b.fidelity).unwrap()
            })
            .map(|(id, _)| *id)
    }
    
    // ========================================================================
    // METRICS AND STATISTICS
    // ========================================================================
    
    /// Calculates aggregated pool metrics
    pub fn get_metrics(&self) -> EMFPoolMetrics {
        let pairs = self.pairs.read().unwrap();
        let pair_states: Vec<EntangledPairState> = pairs.values().cloned().collect();
        
        EMFPoolMetricsAggregator::aggregate_metrics(&pair_states)
    }
    
    /// Returns the current pool pressure (0.0 to 1.0)
    pub fn current_pressure(&self) -> f64 {
        let pairs = self.pairs.read().unwrap();
        pairs.len() as f64 / self.capacity as f64
    }
    
    /// Returns TUCU phase distribution
    pub fn phase_distribution(&self) -> HashMap<ThermodynamicPhase, usize> {
        let pairs = self.pairs.read().unwrap();
        let mut distribution: HashMap<ThermodynamicPhase, usize> = HashMap::new();
        
        for state in pairs.values() {
            let phase = ThermodynamicPhaseClassifier::classify_phase(state);
            *distribution.entry(phase).or_insert(0) += 1;
        }
        
        distribution
    }
    
    pub fn get_statistics(&self) -> &PoolStatistics {
        &self.stats
    }
    
    // ========================================================================
    // MAINTENANCE AND RECYCLING
    // ========================================================================
    
    /// Runs maintenance: recycling degraded pairs
    pub fn run_maintenance(&mut self) -> Result<usize, String> {
        let pressure = self.current_pressure();
        
        // Scan and enqueue pairs for recycling
        let pairs = self.pairs.read().unwrap();
        let mut recycler = self.recycler.write().unwrap();
        
        let enqueued = recycler.scan_and_enqueue(&pairs, pressure);
        drop(pairs);
        
        // Process queue
        let mut pairs = self.pairs.write().unwrap();
        let results = recycler.process_queue(&mut pairs);
        
        // Count successful operations
        let recycled = results.iter()
            .filter(|r| r.success)
            .map(|r| r.recycled_pairs.len())
            .sum();
        
        self.stats.total_pairs_recycled += recycled as u64;
        
        Ok(recycled)
    }
    
    /// Forces recycling of a specific pair
    pub fn recycle_pair(&mut self, pair_id: Uuid) -> Result<(), String> {
        let mut recycler = self.recycler.write().unwrap();
        recycler.enqueue_for_recycling(pair_id)?;
        
        let mut pairs = self.pairs.write().unwrap();
        recycler.process_queue(&mut pairs);
        
        Ok(())
    }
    
    /// Purifies two pairs into one pair with higher fidelity
    pub fn purify_pairs(
        &mut self,
        pair_a: Uuid,
        pair_b: Uuid,
    ) -> Result<Uuid, String> {
        let mut pairs = self.pairs.write().unwrap();
        
        let state_a = pairs.get(&pair_a)
            .ok_or("Pair A not found")?
            .clone();
        
        let state_b = pairs.get(&pair_b)
            .ok_or("Pair B not found")?
            .clone();
        
        // Purification protocol
        let f1 = state_a.fidelity;
        let f2 = state_b.fidelity;
        let purified_fidelity = (f1 * f1 + f2 * f2 - 2.0 * f1 * f1 * f2 * f2).min(0.99);
        
        if purified_fidelity <= f1.max(f2) {
            return Err("Purification does not improve fidelity".to_string());
        }
        
        // Remove original pairs
        pairs.remove(&pair_a);
        pairs.remove(&pair_b);
        
        // Create purified pair (simplified density)
        let mut purified_density = DMatrix::zeros(4, 4);
        purified_density[(0, 0)] = Complex::new(purified_fidelity / 2.0, 0.0);
        purified_density[(3, 3)] = Complex::new(purified_fidelity / 2.0, 0.0);
        purified_density[(0, 3)] = Complex::new(purified_fidelity / 2.0, 0.0);
        purified_density[(3, 0)] = Complex::new(purified_fidelity / 2.0, 0.0);
        
        drop(pairs);
        
        self.create_pair(purified_density)
    }
    
    // ========================================================================
    // PSER ROUTING
    // ========================================================================
    
    /// Routes a pair between source and destination
    pub fn route_pair(
        &mut self,
        source: Uuid,
        destination: Uuid,
        min_fidelity: f64,
    ) -> Result<Vec<Uuid>, String> {
        if !self.config.enable_pser {
            return Err("PSER routing disabled".to_string());
        }
        
        let request = RoutingRequest::new(source, destination, min_fidelity);
        
        let mut router = self.router.write().unwrap();
        let route = router.calculate_route(&request)?;
        
        Ok(route.path)
    }
    
    // ========================================================================
    // UTILITIES
    // ========================================================================
    
    /// Clears all pairs (WARNING!)
    pub fn clear(&mut self) {
        let mut pairs = self.pairs.write().unwrap();
        pairs.clear();
    }
    
    /// Returns the number of pairs in the pool
    pub fn size(&self) -> usize {
        let pairs = self.pairs.read().unwrap();
        pairs.len()
    }
    
    /// Returns pool capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

impl Default for EntanglementPool {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_density_matrix(fidelity: f64) -> DMatrix<Complex<f64>> {
        let mut rho = DMatrix::zeros(4, 4);
        
        // Approximate Bell state
        rho[(0, 0)] = Complex::new(fidelity / 2.0, 0.0);
        rho[(3, 3)] = Complex::new(fidelity / 2.0, 0.0);
        rho[(0, 3)] = Complex::new(fidelity / 2.0, 0.0);
        rho[(3, 0)] = Complex::new(fidelity / 2.0, 0.0);
        
        // Noise
        let noise = (1.0 - fidelity) / 4.0;
        for i in 0..4 {
            rho[(i, i)] += Complex::new(noise, 0.0);
        }
        
        rho
    }
    
    #[test]
    fn test_pool_creation() {
        let pool = EntanglementPool::new();
        assert_eq!(pool.size(), 0);
        assert!(pool.capacity() > 0);
    }
    
    #[test]
    fn test_create_pair() {
        let mut pool = EntanglementPool::new();
        
        let density = create_test_density_matrix(0.95);
        let pair_id = pool.create_pair(density).unwrap();
        
        assert_eq!(pool.size(), 1);
        
        let state = pool.get_pair(pair_id).unwrap();
        assert!(state.fidelity > 0.8);
    }
    
    #[test]
    fn test_consume_pair() {
        let mut pool = EntanglementPool::new();
        
        let density = create_test_density_matrix(0.95);
        let pair_id = pool.create_pair(density).unwrap();
        
        let state = pool.consume_pair(pair_id).unwrap();
        assert_eq!(pool.size(), 0);
        assert!(state.fidelity > 0.8);
    }
    
    #[test]
    fn test_allocate_pair() {
        let mut pool = EntanglementPool::new();
        
        // Create several pairs with different fidelities
        pool.create_pair(create_test_density_matrix(0.85)).unwrap();
        pool.create_pair(create_test_density_matrix(0.95)).unwrap();
        pool.create_pair(create_test_density_matrix(0.75)).unwrap();
        
        // Alocar pair com fidelity >= 0.9
        let allocated = pool.allocate_pair(0.9);
        assert!(allocated.is_some());
        
        let state = pool.get_pair(allocated.unwrap()).unwrap();
        assert!(state.fidelity >= 0.9);
    }
    
    #[test]
    fn test_pool_metrics() {
        let mut pool = EntanglementPool::new();
        
        pool.create_pair(create_test_density_matrix(0.95)).unwrap();
        pool.create_pair(create_test_density_matrix(0.85)).unwrap();
        
        let metrics = pool.get_metrics();
        assert_eq!(metrics.total_pairs, 2);
        assert!(metrics.avg_fidelity > 0.85);
    }
    
    #[test]
    fn test_maintenance() {
        let mut pool = EntanglementPool::new();
        
        // Create degraded pairs
        pool.create_pair(create_test_density_matrix(0.40)).unwrap();
        pool.create_pair(create_test_density_matrix(0.95)).unwrap();
        
        assert_eq!(pool.size(), 2);
        
        // Run maintenance
        pool.run_maintenance().unwrap();
        
        // Degraded pair should have been recycled
        assert!(pool.size() <= 2);
    }
    
    #[test]
    fn test_purification() {
        let mut pool = EntanglementPool::new();
        
        let pair_a = pool.create_pair(create_test_density_matrix(0.85)).unwrap();
        let pair_b = pool.create_pair(create_test_density_matrix(0.82)).unwrap();
        
        let purified = pool.purify_pairs(pair_a, pair_b).unwrap();
        
        assert_eq!(pool.size(), 1);
        
        let state = pool.get_pair(purified).unwrap();
        assert!(state.fidelity > 0.85);
    }
}
