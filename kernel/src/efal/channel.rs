//! Ether field propagation channels

use crate::math::geometric_algebra::GA3D;
use crate::math::fiber_bundle::ConnectionType;
use std::time::Duration;

/// Quantum propagation channel
#[derive(Clone)]
pub struct Channel {
    pub id: String,
    pub source: String,      // Source defect ID
    pub target: String,      // Target defect ID
    pub channel_type: ChannelType,
    pub path: Vec<GA3D>,     // Spatial path
    pub status: ChannelStatus,
    pub metrics: ChannelMetrics,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelType {
    Optical,           // Optical fiber or free space
    Microwave,         // Microwave guide (superconductors)
    Phononic,          // Acoustic waves (neutral atoms)
    IonMotional,       // Vibrational modes (trapped ions)
    Virtual,           // Connection via entanglement
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelStatus {
    Idle,
    Active,
    Degraded,
    Failed,
}

#[derive(Clone)]
pub struct ChannelMetrics {
    pub fidelity: f64,            // Transmission fidelity
    pub latency: Duration,        // Measured latency
    pub attenuation_db: f64,      // Attenuation (dB)
    pub coherence_time: Duration, // Coherence time
}

impl Channel {
    pub fn new(
        source: String,
        target: String,
        channel_type: ChannelType,
        path: Vec<GA3D>,
    ) -> Self {
        Channel {
            id: format!("ch_{}_{}", source, target),
            source,
            target,
            channel_type,
            path,
            status: ChannelStatus::Idle,
            metrics: ChannelMetrics::default(),
        }
    }
    
    /// Activates the channel
    pub fn activate(&mut self) -> Result<(), String> {
        if self.status == ChannelStatus::Failed {
            return Err("Cannot activate failed channel".to_string());
        }
        
        self.status = ChannelStatus::Active;
        Ok(())
    }
    
    /// Deactivates the channel
    pub fn deactivate(&mut self) {
        self.status = ChannelStatus::Idle;
    }
    
    /// Computes the total path length
    pub fn path_length(&self) -> f64 {
        if self.path.len() < 2 {
            return 0.0;
        }
        
        self.path.windows(2)
            .map(|window| window[0].distance(&window[1]))
            .sum()
    }
    
    /// Updates metrics based on telemetry
    pub fn update_metrics(&mut self, new_metrics: ChannelMetrics) {
        self.metrics = new_metrics;
        
        // Updates status based on fidelity
        if self.metrics.fidelity < 0.5 {
            self.status = ChannelStatus::Failed;
        } else if self.metrics.fidelity < 0.9 {
            self.status = ChannelStatus::Degraded;
        }
    }
    
    /// Converts to the fiber bundle's ConnectionType
    pub fn to_connection_type(&self) -> ConnectionType {
        match self.channel_type {
            ChannelType::Optical => ConnectionType::Optical,
            ChannelType::Microwave => ConnectionType::Microwave,
            ChannelType::Phononic => ConnectionType::Phononic,
            ChannelType::IonMotional => ConnectionType::Microwave, // Approximation
            ChannelType::Virtual => ConnectionType::Virtual,
        }
    }
}

impl Default for ChannelMetrics {
    fn default() -> Self {
        ChannelMetrics {
            fidelity: 0.99,
            latency: Duration::from_nanos(100),
            attenuation_db: 0.1,
            coherence_time: Duration::from_micros(100),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_channel_creation() {
        let path = vec![
            GA3D::new(0.0, 0.0, 0.0),
            GA3D::new(1.0, 0.0, 0.0),
        ];
        
        let channel = Channel::new(
            "dt_0".to_string(),
            "dt_1".to_string(),
            ChannelType::Optical,
            path,
        );
        
        assert_eq!(channel.path_length(), 1.0);
        assert_eq!(channel.status, ChannelStatus::Idle);
    }
}
