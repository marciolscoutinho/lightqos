//! Entangled State Pool

use std::time::{Duration, Instant};

/// Bell pair (EPR state)
#[derive(Clone)]
pub struct BellPair {
    pub id: String,
    pub qubit_a: String,
    pub qubit_b: String,
    pub state_type: BellState,
    pub fidelity: f64,
    pub creation_time: Instant,
    pub lifetime: Duration,
}

#[derive(Debug, Clone, Copy)]
pub enum BellState {
    PhiPlus,   // |Φ+⟩ = (|00⟩ + |11⟩)/√2
    PhiMinus,  // |Φ-⟩ = (|00⟩ - |11⟩)/√2
    PsiPlus,   // |Ψ+⟩ = (|01⟩ + |10⟩)/√2
    PsiMinus,  // |Ψ-⟩ = (|01⟩ - |10⟩)/√2
}

impl BellPair {
    pub fn create_via_route(
        qubit_a: &str,
        qubit_b: &str,
        _route: crate::emf::pser_routing::PSERRoute,
        target_fidelity: f64,
    ) -> Result<Self, crate::emf::EMFError> {
        // Simulated physical creation
        // On real hardware, this would delegate to the driver
        
        Ok(BellPair {
            id: format!("bell_{}_{}", qubit_a, qubit_b),
            qubit_a: qubit_a.to_string(),
            qubit_b: qubit_b.to_string(),
            state_type: BellState::PhiPlus,
            fidelity: target_fidelity * 0.99, // Realistic fidelity
            creation_time: Instant::now(),
            lifetime: Duration::from_millis(10), // Coherence time
        })
    }
    
    /// Checks whether the pair is still coherent
    pub fn is_coherent(&self) -> bool {
        self.creation_time.elapsed() < self.lifetime
    }
    
    /// Estimates ergotropy (extractable work)
    pub fn ergotropy(&self) -> f64 {
        // W_max ≈ 2F - 1 (normalized)
        2.0 * self.fidelity - 1.0
    }
}

/// GHZ state (multipartite)
pub struct GHZState {
    pub id: String,
    pub qubits: Vec<String>,
    pub fidelity: f64,
    pub creation_time: Instant,
}

impl GHZState {
    pub fn new(qubits: Vec<String>, fidelity: f64) -> Self {
        GHZState {
            id: format!("ghz_{}", qubits.len()),
            qubits,
            fidelity,
            creation_time: Instant::now(),
        }
    }
}
