//! EMF - Entangled Memory Fabric
//! 
//! Manages entangled pairs as a shared-memory resource,
//! implementing TUCU’s Generation–Radiation cycles.

pub mod entanglement_pool;
pub mod pser_routing;
pub mod metrics;
pub mod recycler;

use std::collections::HashMap;
use crate::efal::EtherField;

/// Global pool of entangled states
pub struct EntanglementFabric {
    /// Available Bell pairs (high ergotropy)
    pub bell_pairs: HashMap<String, entanglement_pool::BellPair>,
    
    /// Multipartite GHZ states
    pub ghz_states: Vec<entanglement_pool::GHZState>,
    
    /// Thermodynamic metrics for each resource
    pub metrics: metrics::QuantumMetrics,
    
    /// Recycler for degraded entanglement
    pub recycler: recycler::EntanglementRecycler,
    
    /// Reference to the ether field (for routing)
    ether_field: *const EtherField,
}

impl EntanglementFabric {
    pub fn new(ether_field: &EtherField) -> Self {
        EntanglementFabric {
            bell_pairs: HashMap::new(),
            ghz_states: Vec::new(),
            metrics: metrics::QuantumMetrics::new(),
            recycler: recycler::EntanglementRecycler::new(),
            ether_field: ether_field as *const _,
        }
    }
    
    /// Allocates a Bell pair between two defects (qubits)
    /// Returns the pair ID or an error if it is not possible
    pub fn allocate_bell_pair(
        &mut self,
        defect_a: &str,
        defect_b: &str,
        min_fidelity: f64,
    ) -> Result<String, EMFError> {
        // Checks whether there is already a pre-created pair between these defects
        let existing_key = format!("{}_{}", defect_a, defect_b);
        
        if let Some(pair) = self.bell_pairs.get(&existing_key) {
            // Checks ergotropy (work potential)
            let ergotropy = self.metrics.compute_ergotropy(pair);
            
            if ergotropy >= Self::ergotropy_threshold(min_fidelity) {
                return Ok(existing_key);
            } else {
                // Mark for recycling (Radiation 1- to 4-)
                self.recycler.mark_for_recycling(&existing_key);
            }
        }
        
        // Creates a new pair (Generation 4+ to 1+)
        let new_pair = self.generate_bell_pair(defect_a, defect_b, min_fidelity)?;
        let pair_id = format!("bell_{}_{}", defect_a, defect_b);
        
        self.bell_pairs.insert(pair_id.clone(), new_pair);
        
        Ok(pair_id)
    }
    
    /// Generates a new Bell pair via the PSER protocol
    fn generate_bell_pair(
        &self,
        defect_a: &str,
        defect_b: &str,
        target_fidelity: f64,
    ) -> Result<entanglement_pool::BellPair, EMFError> {
        // Uses PSER routing to find an optimal path
        let ether = unsafe { &*self.ether_field };
        
        let route = pser_routing::compute_optimal_route(
            defect_a,
            defect_b,
            &ether.geometry,
            &ether.topology,
        )?;
        
        // Creates the physical pair via the hardware driver
        let pair = entanglement_pool::BellPair::create_via_route(
            defect_a,
            defect_b,
            route,
            target_fidelity,
        )?;
        
        Ok(pair)
    }
    
    /// Recycling cycle (Radiation → Inertia → Generation)
    pub fn recycle_degraded_pairs(&mut self) {
        let to_recycle = self.recycler.get_marked_pairs();
        
        for pair_id in to_recycle {
            if let Some(pair) = self.bell_pairs.remove(&pair_id) {
                // Measures the final entropy (complete Radiation)
                let final_entropy = self.metrics.compute_entropy(&pair);
                
                // Returns to the inertia state (0=)
                // Statistics for The Light to learn from
                self.metrics.log_recycling(pair_id, final_entropy);
            }
        }
    }
    
    /// Ergotropy threshold based on the desired fidelity
    fn ergotropy_threshold(fidelity: f64) -> f64 {
        // For a Bell pair:
        // W_max ≈ (2F - 1) (normalized)
        // Where F is the fidelity
        2.0 * fidelity - 1.0
    }
}

#[derive(Debug)]
pub enum EMFError {
    InsufficientQuality,
    RoutingFailed(String),
    HardwareError(String),
}
