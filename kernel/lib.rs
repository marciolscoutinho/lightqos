//! LightQOS Kernel - Quantum Operational System
//! 
//! Based on the principles of the Unified Theory of Universal Consciousness (TUCU)

#![warn(missing_docs)]
#![warn(rustdoc::broken_intra_doc_links)]

pub mod efal;
pub mod emf;
pub mod tlm;
pub mod hio;
pub mod math;
pub mod utils;

// Main re-exports
pub use efal::EtherField;
pub use emf::EntanglementFabric;
pub use tlm::TemporalLayerManager;
pub use hio::HolographicIO;

/// Kernel version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initializes the LightQOS kernel
pub fn init() -> KernelHandle {
    KernelHandle::new()
}

/// Main kernel handle
pub struct KernelHandle {
    /// Ether Field
    pub ether_field: EtherField,
    
    /// Entanglement Fabric
    pub emf: EntanglementFabric,
    
    /// Temporal Manager
    pub tlm: TemporalLayerManager,
    
    /// Holographic I/O
    pub hio: HolographicIO,
}

impl KernelHandle {
    fn new() -> Self {
        // Default configuration
        let config = efal::HardwareConfig {
            platform: "simulator".to_string(),
            num_qubits: 5,
            connectivity: vec![(0, 1), (1, 2), (2, 3), (3, 4)],
            coherence_times: vec![1e-3; 5],
        };
        
        let ether_field = EtherField::new(&config);
        let emf = EntanglementFabric::new(&ether_field);
        let tlm = TemporalLayerManager::new();
        let hio = HolographicIO::new();
        
        KernelHandle {
            ether_field,
            emf,
            tlm,
            hio,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_kernel_init() {
        let kernel = init();
        assert_eq!(kernel.ether_field.defects.len(), 0);
    }
}
