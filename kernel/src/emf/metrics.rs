//! Thermodynamic Metrics of Entanglement

use super::entanglement_pool::BellPair;

pub struct QuantumMetrics {
    recycling_log: Vec<RecyclingEvent>,
}

impl QuantumMetrics {
    pub fn new() -> Self {
        QuantumMetrics {
            recycling_log: Vec::new(),
        }
    }
    
    /// Computes the ergotropy of a Bell pair
    pub fn compute_ergotropy(&self, pair: &BellPair) -> f64 {
        pair.ergotropy()
    }
    
    /// Computes the (approximate) von Neumann entropy
    pub fn compute_entropy(&self, pair: &BellPair) -> f64 {
        // S = -Tr(ρ log ρ)
        // For a Bell pair: S ≈ 1 - F (approximation)
        1.0 - pair.fidelity
    }
    
    /// Logs a recycling event
    pub fn log_recycling(&mut self, pair_id: String, final_entropy: f64) {
        self.recycling_log.push(RecyclingEvent {
            pair_id,
            timestamp: std::time::Instant::now(),
            final_entropy,
        });
    }
}

struct RecyclingEvent {
    pair_id: String,
    timestamp: std::time::Instant,
    final_entropy: f64,
}
