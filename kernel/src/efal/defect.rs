//! Topological Defects (Stable Qubits)

use crate::math::geometric_algebra::GA3D;
use crate::math::octave_algebra::OctavePosition;
use std::time::Duration;

/// Topological defect in the ether field
#[derive(Clone)]
pub struct TopologicalDefect {
    pub id: String,
    pub position: GA3D,
    pub defect_type: DefectType,
    pub octave_state: OctavePosition,
    pub status: DefectStatus,
    pub metrics: DefectMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefectType {
    Qubit,              // Standard binary qubit
    Qudit(usize),       // Qudit of dimension d
    NetworkNode,        // Quantum network node
    AncillaQubit,       // Auxiliary qubit (error correction)
    MeasurementPoint,   // Measurement point
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefectStatus {
    Idle,
    InUse,
    Entangled,
    Measured,
    Failed,
}

#[derive(Clone)]
pub struct DefectMetrics {
    pub T1: Duration,           // Relaxation time
    pub T2: Duration,           // Decoherence time
    pub gate_fidelity: f64,     // Gate fidelity
    pub readout_fidelity: f64,  // Readout fidelity
}

impl TopologicalDefect {
    pub fn new(position: GA3D, defect_type: DefectType) -> Self {
        TopologicalDefect {
            id: format!("dt_{}", uuid::Uuid::new_v4()),
            position,
            defect_type,
            octave_state: OctavePosition::Inertia0, // Starts at rest
            status: DefectStatus::Idle,
            metrics: DefectMetrics::default(),
        }
    }
    
    /// Hilbert space dimension
    pub fn hilbert_dimension(&self) -> usize {
        match self.defect_type {
            DefectType::Qubit => 2,
            DefectType::Qudit(d) => d,
            DefectType::NetworkNode => 2,
            DefectType::AncillaQubit => 2,
            DefectType::MeasurementPoint => 1,
        }
    }
    
    /// Allocates the defect for use
    pub fn allocate(&mut self) -> Result<(), String> {
        if self.status == DefectStatus::Failed {
            return Err("Cannot allocate failed defect".to_string());
        }
        
        self.status = DefectStatus::InUse;
        Ok(())
    }

    /// Releases the defect
    pub fn release(&mut self) {
        self.status = DefectStatus::Idle;
        self.octave_state = OctavePosition::Inertia0;
    }

    /// Marks as entangled
    pub fn entangle(&mut self) {
        self.status = DefectStatus::Entangled;
    }

    /// Octave transition (for operations)
    pub fn transition_to_octave(&mut self, target: OctavePosition) {
        self.octave_state = target;
    }

    /// Checks whether it is available
    pub fn is_available(&self) -> bool {
        matches!(self.status, DefectStatus::Idle | DefectStatus::InUse)
    }
}

impl Default for DefectMetrics {
    fn default() -> Self {
        DefectMetrics {
            T1: Duration::from_micros(100),
            T2: Duration::from_micros(50),
            gate_fidelity: 0.999,
            readout_fidelity: 0.995,
        }
    }
}

// External dependency (add to Cargo.toml)
mod uuid {
    pub struct Uuid;

    impl Uuid {
        pub fn new_v4() -> String {
            use std::time::SystemTime;

            let now = SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();

            format!("{:x}", now)
        }
    }
}
