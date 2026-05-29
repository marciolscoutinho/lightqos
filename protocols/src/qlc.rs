// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// qlc.rs — QLC Protocol — Quantum Link Control
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 25-07-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use num_complex::Complex64;
use uuid::Uuid;
use rand::Rng;

// ============================================================================
// QUANTUM LIGHT COMMUNICATION
// ============================================================================

/// QLC protocol
#[derive(Debug, Clone)]
pub struct QuantumLightCommunication {
    /// Unique ID
    pub id: Uuid,
    
    /// Active light channels
    pub channels: Vec<LightChannel>,
    
    /// Maximum channel capacity
    pub max_channels: usize,
    
    /// Statistics
    pub stats: CommunicationStats,
}

impl QuantumLightCommunication {
    /// Creates a new QLC protocol
    pub fn new(max_channels: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            channels: Vec::new(),
            max_channels,
            stats: CommunicationStats::default(),
        }
    }
    
    /// Creates a light channel
    pub fn create_channel(
        &mut self,
        octave: u8,
        bandwidth: f64,
    ) -> Result<Uuid, String> {
        if self.channels.len() >= self.max_channels {
            return Err("Maximum channels reached".to_string());
        }
        
        assert!(octave >= 1 && octave <= 10, "Octave must be 1-10");
        
        let channel = LightChannel::new(octave, bandwidth);
        let id = channel.id;
        
        self.channels.push(channel);
        
        Ok(id)
    }
    
    /// Transmits a qubit through a channel
    pub fn transmit_qubit(
        &mut self,
        channel_id: &Uuid,
        qubit: QuantumBit,
    ) -> Result<TransmissionResult, String> {
        let channel = self.channels.iter_mut()
            .find(|c| &c.id == channel_id)
            .ok_or("Channel not found")?;
        
        // Encode the qubit into photons
        let photons = self.encode_qubit_to_photons(qubit, channel.octave);
        
        // Transmit through the channel
        let result = channel.transmit(photons)?;
        
        // Update statistics
        self.stats.total_transmissions += 1;
        if result.success {
            self.stats.successful_transmissions += 1;
        }
        self.stats.total_qubits_transmitted += 1;
        
        Ok(result)
    }
    
    /// Encodes a qubit into photons
    fn encode_qubit_to_photons(&self, qubit: QuantumBit, octave: u8) -> Vec<Photon> {
        let num_photons = 1; // Simplification: 1 photon per qubit
        
        let mut photons = Vec::new();
        
        for _ in 0..num_photons {
            let photon = Photon {
                octave,
                frequency: Self::octave_frequency(octave),
                wavelength: Self::octave_wavelength(octave),
                polarization: qubit.polarization,
                phase: qubit.phase,
                amplitude: qubit.amplitude,
            };
            
            photons.push(photon);
        }
        
        photons
    }
    
    /// Octave frequency (Hz)
    fn octave_frequency(octave: u8) -> f64 {
        2.0_f64.powi((octave - 1) as i32)
    }
    
    /// Octave wavelength (m)
    fn octave_wavelength(octave: u8) -> f64 {
        const C: f64 = 299_792_458.0; // Velocidade da luz
        C / Self::octave_frequency(octave)
    }
    
    /// Gets a channel by ID
    pub fn get_channel(&self, id: &Uuid) -> Option<&LightChannel> {
        self.channels.iter().find(|c| &c.id == id)
    }
    
    /// Removes a channel
    pub fn close_channel(&mut self, id: &Uuid) -> Option<LightChannel> {
        if let Some(pos) = self.channels.iter().position(|c| &c.id == id) {
            Some(self.channels.remove(pos))
        } else {
            None
        }
    }
}

// ============================================================================
// LIGHT CHANNEL
// ============================================================================

/// Quantum light channel
#[derive(Debug, Clone)]
pub struct LightChannel {
    /// Unique ID
    pub id: Uuid,
    
    /// Electromagnetic octave (1-10)
    pub octave: u8,
    
    /// Bandwidth (Hz)
    pub bandwidth: f64,
    
    /// Loss rate (dB/km)
    pub loss_rate: f64,
    
    /// Channel fidelity
    pub fidelity: f64,
    
    /// Channel distance (km)
    pub distance: f64,
    
    /// Is the channel active?
    pub active: bool,
}

impl LightChannel {
    pub fn new(octave: u8, bandwidth: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            octave,
            bandwidth,
            loss_rate: 0.2, // Typical 0.2 dB/km
            fidelity: 0.99,
            distance: 1.0, // Default 1 km
            active: true,
        }
    }
    
    /// Transmits photons through the channel
    pub fn transmit(&mut self, photons: Vec<Photon>) -> Result<TransmissionResult, String> {
        if !self.active {
            return Err("Channel is inactive".to_string());
        }
        
        // Calculate attenuation loss
        let loss_db = self.loss_rate * self.distance;
        let loss_factor = 10.0_f64.powf(-loss_db / 10.0);
        
        // Fidelity after transmission
        let final_fidelity = self.fidelity * loss_factor;
        
        // Error rate
        let error_rate = 1.0 - final_fidelity;
        
        // Check whether the transmission was successful
        let mut rng = rand::thread_rng();
        let success = rng.gen::<f64>() > error_rate;
        
        Ok(TransmissionResult {
            success,
            final_fidelity,
            photons_transmitted: photons.len(),
            octave_used: self.octave,
            distance: self.distance,
        })
    }
}

// ============================================================================
// PHOTON
// ============================================================================

/// Quantum photon
#[derive(Debug, Clone, Copy)]
pub struct Photon {
    /// Octave (1-10)
    pub octave: u8,
    
    /// Frequency (Hz)
    pub frequency: f64,
    
    /// Wavelength (m)
    pub wavelength: f64,
    
    /// Polarization (angle in radians)
    pub polarization: f64,
    
    /// Phase (radians)
    pub phase: f64,
    
    /// Complex amplitude
    pub amplitude: Complex64,
}

// ============================================================================
// QUANTUM BIT
// ============================================================================

/// Qubit to be transmitted
#[derive(Debug, Clone, Copy)]
pub struct QuantumBit {
    /// Polarization (0 = |0⟩, π/2 = |1⟩)
    pub polarization: f64,
    
    /// Relative phase
    pub phase: f64,
    
    /// Amplitude
    pub amplitude: Complex64,
}

impl QuantumBit {
    /// Creates the |0⟩ qubit
    pub fn zero() -> Self {
        Self {
            polarization: 0.0,
            phase: 0.0,
            amplitude: Complex64::new(1.0, 0.0),
        }
    }
    
    /// Creates the |1⟩ qubit
    pub fn one() -> Self {
        Self {
            polarization: std::f64::consts::PI / 2.0,
            phase: 0.0,
            amplitude: Complex64::new(1.0, 0.0),
        }
    }
    
    /// Creates the |+⟩ qubit
    pub fn plus() -> Self {
        Self {
            polarization: std::f64::consts::PI / 4.0,
            phase: 0.0,
            amplitude: Complex64::new(1.0 / 2.0_f64.sqrt(), 0.0),
        }
    }
}

// ============================================================================
// TRANSMISSION RESULT
// ============================================================================

/// Transmission result
#[derive(Debug, Clone)]
pub struct TransmissionResult {
    /// Was the transmission successful?
    pub success: bool,
    
    /// Final fidelity
    pub final_fidelity: f64,
    
    /// Number of transmitted photons
    pub photons_transmitted: usize,
    
    /// Octave used
    pub octave_used: u8,
    
    /// Distance traveled (km)
    pub distance: f64,
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Communication statistics
#[derive(Debug, Clone, Default)]
pub struct CommunicationStats {
    /// Total transmissions
    pub total_transmissions: usize,
    
    /// Successful transmissions
    pub successful_transmissions: usize,
    
    /// Total transmitted qubits
    pub total_qubits_transmitted: usize,
}

impl CommunicationStats {
    /// Success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_transmissions == 0 {
            return 0.0;
        }
        self.successful_transmissions as f64 / self.total_transmissions as f64
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_qlc_creation() {
        let qlc = QuantumLightCommunication::new(10);
        assert_eq!(qlc.max_channels, 10);
        assert_eq!(qlc.channels.len(), 0);
    }
    
    #[test]
    fn test_create_channel() {
        let mut qlc = QuantumLightCommunication::new(10);
        let result = qlc.create_channel(5, 1.0e9);
        
        assert!(result.is_ok());
        assert_eq!(qlc.channels.len(), 1);
    }
    
    #[test]
    fn test_transmit_qubit() {
        let mut qlc = QuantumLightCommunication::new(10);
        let channel_id = qlc.create_channel(4, 1.0e9).unwrap();
        
        let qubit = QuantumBit::zero();
        let result = qlc.transmit_qubit(&channel_id, qubit);
        
        assert!(result.is_ok());
    }
}
