// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// qblox_driver.rs — Qblox Driver — QCM/QRM pulse control hardware integration
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 20-01-2026
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Types and constants
// ============================================================================

/// Maximum Qblox ADC/DAC sampling rate (1 GSPS)
pub const QBLOX_SAMPLE_RATE_HZ: f64 = 1.0e9;

/// NCO phase resolution (32 bits → 2³² degrees)
pub const QBLOX_PHASE_RESOLUTION_BITS: u32 = 32;

/// Maximum number of sequencers per module
pub const MAX_SEQUENCERS: usize = 6;

/// Maximum amplitude in mV (16-bit DAC output)
pub const MAX_AMPLITUDE_MV: f64 = 500.0;

// ============================================================================
// Enums
// ============================================================================

/// Qblox module type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum QbloxModuleType {
    /// QCM — Qubit Control Module (microwave, 4 outputs)
    QCM,
    /// QCM-RF — QCM with integrated RF up-conversion
    QCM_RF,
    /// QRM — Qubit Readout Module (1 input, 1 output)
    QRM,
    /// QRM-RF — QRM with integrated RF down-conversion
    QRM_RF,
}

/// Module status
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModuleStatus {
    Idle,
    Armed,
    Running,
    Error(QbloxError),
}

/// Qblox-specific errors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum QbloxError {
    SequencerTimeout,
    FIFOOverflow,
    CalibrationFailed,
    ConnectionLost,
    InvalidPulseParams,
}

// ============================================================================
// Pulse — Quantum pulse definition
// ============================================================================

/// Quantum pulse to be sent to the Qblox sequencer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QbloxPulse {
    /// Unique pulse identifier
    pub id: String,
    /// Local oscillator frequency (Hz)
    pub lo_frequency_hz: f64,
    /// Relative NCO frequency (Hz)
    pub nco_frequency_hz: f64,
    /// I amplitude (em escala -1.0 a 1.0)
    pub amplitude_i: f64,
    /// Q amplitude (em escala -1.0 a 1.0)
    pub amplitude_q: f64,
    /// Duration in ns (multiple of 4)
    pub duration_ns: u64,
    /// Initial phase in degrees (0.0-360.0)
    pub phase_deg: f64,
    /// Pulse envelope shape
    pub envelope: PulseEnvelope,
    /// Sequencer index (0-5)
    pub sequencer: usize,
}

/// Pulse envelope shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PulseEnvelope {
    /// Rectangular envelope (square-wave pulse)
    Square,
    /// Gaussian with sigma in ns
    Gaussian { sigma_ns: f64 },
    /// DRAG — Derivative Removal via Adiabatic Gate
    DRAG { sigma_ns: f64, beta: f64 },
    /// Cosine raised (COSINE)
    CosineBell,
    /// Arbitrary waveform (I/Q samples)
    Arbitrary { samples_i: Vec<f64>, samples_q: Vec<f64> },
}

impl QbloxPulse {
    /// Creates a simple rectangular pulse
    pub fn rectangular(
        sequencer: usize,
        lo_freq: f64,
        nco_freq: f64,
        amp: f64,
        duration_ns: u64,
    ) -> Self {
        let duration_ns = (duration_ns / 4) * 4; // multiple of 4
        Self {
            id: Uuid::new_v4().to_string(),
            lo_frequency_hz: lo_freq,
            nco_frequency_hz: nco_freq,
            amplitude_i: amp,
            amplitude_q: 0.0,
            duration_ns,
            phase_deg: 0.0,
            envelope: PulseEnvelope::Square,
            sequencer: sequencer.min(MAX_SEQUENCERS - 1),
        }
    }

    /// Creates a Gaussian pulse
    pub fn gaussian(
        sequencer: usize,
        lo_freq: f64,
        nco_freq: f64,
        amp: f64,
        sigma_ns: f64,
        duration_ns: u64,
    ) -> Self {
        let mut p = Self::rectangular(sequencer, lo_freq, nco_freq, amp, duration_ns);
        p.envelope = PulseEnvelope::Gaussian { sigma_ns };
        p
    }

    /// Creates a DRAG pulse to reduce leakage
    pub fn drag(
        sequencer: usize,
        lo_freq: f64,
        nco_freq: f64,
        amp: f64,
        sigma_ns: f64,
        beta: f64,
        duration_ns: u64,
    ) -> Self {
        let mut p = Self::gaussian(sequencer, lo_freq, nco_freq, amp, sigma_ns, duration_ns);
        p.envelope = PulseEnvelope::DRAG { sigma_ns, beta };
        p
    }

    /// X gate (π-pulse) — transition frequency 5 GHz, IBM example
    pub fn x_gate(qubit_freq_hz: f64, sequencer: usize) -> Self {
        Self::drag(sequencer, qubit_freq_hz, 0.0, 0.5, 8.0, 0.5, 40)
    }

    /// H gate (Hadamard) — π/2 in X followed by π in Y
    pub fn h_gate(qubit_freq_hz: f64, sequencer: usize) -> Self {
        Self::drag(sequencer, qubit_freq_hz, 0.0, 0.25, 8.0, 0.5, 40)
    }

    /// Verifies whether the parameters are valid for Qblox hardware
    pub fn validate(&self) -> Result<(), QbloxError> {
        if self.amplitude_i.abs() > 1.0 || self.amplitude_q.abs() > 1.0 {
            return Err(QbloxError::InvalidPulseParams);
        }
        if self.duration_ns % 4 != 0 {
            return Err(QbloxError::InvalidPulseParams);
        }
        if self.sequencer >= MAX_SEQUENCERS {
            return Err(QbloxError::InvalidPulseParams);
        }
        Ok(())
    }
}

// ============================================================================
// QbloxSequencer — one of the 6 sequencers per module
// ============================================================================

/// Internal sequencer of a Qblox module
#[derive(Debug)]
pub struct QbloxSequencer {
    pub index: usize,
    pub enabled: bool,
    pub nco_frequency_hz: f64,
    pub lo_frequency_hz: f64,
    pub pulse_queue: Vec<QbloxPulse>,
    pub total_pulses_sent: usize,
}

impl QbloxSequencer {
    pub fn new(index: usize) -> Self {
        Self {
            index,
            enabled: false,
            nco_frequency_hz: 0.0,
            lo_frequency_hz: 0.0,
            pulse_queue: Vec::new(),
            total_pulses_sent: 0,
        }
    }

    /// Adds a pulse to the queue
    pub fn queue_pulse(&mut self, pulse: QbloxPulse) -> Result<(), QbloxError> {
        pulse.validate()?;
        self.nco_frequency_hz = pulse.nco_frequency_hz;
        self.lo_frequency_hz = pulse.lo_frequency_hz;
        self.pulse_queue.push(pulse);
        Ok(())
    }

    /// "Executes" the pulse queue (simulated)
    pub fn flush(&mut self) -> usize {
        let n = self.pulse_queue.len();
        self.total_pulses_sent += n;
        self.pulse_queue.clear();
        n
    }

    /// Calculates the total sequence duration in ns
    pub fn total_duration_ns(&self) -> u64 {
        self.pulse_queue.iter().map(|p| p.duration_ns).sum()
    }
}

// ============================================================================
// QbloxModule — QCM or QRM module
// ============================================================================

/// Qblox module (QCM or QRM)
#[derive(Debug)]
pub struct QbloxModule {
    pub slot: u8,
    pub module_type: QbloxModuleType,
    pub serial_number: String,
    pub sequencers: [QbloxSequencer; MAX_SEQUENCERS],
    pub status: ModuleStatus,
    pub temperature_celsius: f64,
}

impl QbloxModule {
    pub fn new(slot: u8, module_type: QbloxModuleType) -> Self {
        Self {
            slot,
            module_type,
            serial_number: format!("QB-{:04X}", slot as u32 * 0x1234),
            sequencers: std::array::from_fn(|i| QbloxSequencer::new(i)),
            status: ModuleStatus::Idle,
            temperature_celsius: 25.0,
        }
    }

    /// Sends a pulse to the specified sequencer
    pub fn send_pulse(&mut self, pulse: QbloxPulse) -> Result<(), QbloxError> {
        let seq_idx = pulse.sequencer;
        self.sequencers[seq_idx].queue_pulse(pulse)?;
        Ok(())
    }

    /// Arms all sequencers (prepares for trigger)
    pub fn arm(&mut self) {
        for seq in &mut self.sequencers {
            seq.enabled = true;
        }
        self.status = ModuleStatus::Armed;
    }

    /// Triggers all armed sequencers
    pub fn trigger(&mut self) -> usize {
        self.status = ModuleStatus::Running;
        let total: usize = self.sequencers.iter_mut()
            .filter(|s| s.enabled && !s.pulse_queue.is_empty())
            .map(|s| s.flush())
            .sum();
        self.status = ModuleStatus::Idle;
        total
    }

    /// Full module reset
    pub fn reset(&mut self) {
        for seq in &mut self.sequencers {
            seq.pulse_queue.clear();
            seq.enabled = false;
        }
        self.status = ModuleStatus::Idle;
    }
}

// ============================================================================
// QbloxCluster — rack with multiple modules
// ============================================================================

/// Qblox cluster — up to 20 modules in one chassis
#[derive(Debug)]
pub struct QbloxCluster {
    pub cluster_id: String,
    pub ip_address: String,
    pub modules: HashMap<u8, QbloxModule>,
    pub connected: bool,
    pub firmware_version: String,
}

impl QbloxCluster {
    /// Creates a cluster (simulation mode)
    pub fn new_simulated(ip: impl Into<String>) -> Self {
        Self {
            cluster_id: Uuid::new_v4().to_string(),
            ip_address: ip.into(),
            modules: HashMap::new(),
            connected: true,
            firmware_version: "0.6.2".into(),
        }
    }

    /// Adds a module to the cluster
    pub fn add_module(&mut self, slot: u8, module_type: QbloxModuleType) {
        self.modules.insert(slot, QbloxModule::new(slot, module_type));
    }

    /// Configures a typical cluster: 4 QCM + 2 QRM
    pub fn configure_standard_6_qubit(&mut self) {
        self.add_module(1, QbloxModuleType::QCM_RF);
        self.add_module(2, QbloxModuleType::QCM_RF);
        self.add_module(3, QbloxModuleType::QCM_RF);
        self.add_module(4, QbloxModuleType::QCM_RF);
        self.add_module(5, QbloxModuleType::QRM_RF);
        self.add_module(6, QbloxModuleType::QRM_RF);
    }

    /// Sends a pulse to the correct module
    pub fn send_pulse(&mut self, slot: u8, pulse: QbloxPulse) -> Result<(), QbloxError> {
        let module = self.modules.get_mut(&slot)
            .ok_or(QbloxError::ConnectionLost)?;
        module.send_pulse(pulse)
    }

    /// Triggers all modules simultaneously (hardware-synchronized)
    pub fn trigger_all(&mut self) -> usize {
        self.modules.values_mut()
            .map(|m| { m.arm(); m.trigger() })
            .sum()
    }

    /// Cluster statistics
    pub fn stats(&self) -> ClusterStats {
        ClusterStats {
            num_modules: self.modules.len(),
            connected: self.connected,
            firmware: self.firmware_version.clone(),
            total_sequencers: self.modules.len() * MAX_SEQUENCERS,
        }
    }
}

#[derive(Debug)]
pub struct ClusterStats {
    pub num_modules: usize,
    pub connected: bool,
    pub firmware: String,
    pub total_sequencers: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pulse_validation_valid() {
        let pulse = QbloxPulse::rectangular(0, 5e9, 0.0, 0.5, 40);
        assert!(pulse.validate().is_ok());
    }

    #[test]
    fn test_pulse_validation_bad_amplitude() {
        let mut pulse = QbloxPulse::rectangular(0, 5e9, 0.0, 0.5, 40);
        pulse.amplitude_i = 1.5;
        assert_eq!(pulse.validate(), Err(QbloxError::InvalidPulseParams));
    }

    #[test]
    fn test_pulse_duration_alignment() {
        let pulse = QbloxPulse::rectangular(0, 5e9, 0.0, 0.5, 37);
        assert_eq!(pulse.duration_ns % 4, 0);
    }

    #[test]
    fn test_x_gate_pulse() {
        let gate = QbloxPulse::x_gate(5.1e9, 0);
        assert!(gate.validate().is_ok());
        assert_eq!(gate.duration_ns, 40);
    }

    #[test]
    fn test_module_send_and_trigger() {
        let mut module = QbloxModule::new(1, QbloxModuleType::QCM_RF);
        let pulse = QbloxPulse::x_gate(5.1e9, 0);
        module.send_pulse(pulse).unwrap();
        module.arm();
        let sent = module.trigger();
        assert_eq!(sent, 1);
    }

    #[test]
    fn test_cluster_standard_config() {
        let mut cluster = QbloxCluster::new_simulated("192.168.1.100");
        cluster.configure_standard_6_qubit();
        let stats = cluster.stats();
        assert_eq!(stats.num_modules, 6);
        assert!(stats.connected);
    }

    #[test]
    fn test_cluster_trigger_all() {
        let mut cluster = QbloxCluster::new_simulated("192.168.1.100");
        cluster.configure_standard_6_qubit();
        let pulse = QbloxPulse::x_gate(5.1e9, 0);
        cluster.send_pulse(1, pulse).unwrap();
        let sent = cluster.trigger_all();
        assert_eq!(sent, 1);
    }
}
