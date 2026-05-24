// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// zurich_driver.rs — Zurich Instruments Driver — SHFQA/SHFSG/HDAWG integration
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 20-10-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Constantes
// ============================================================================

/// SHFQA sampling rate (2 GSPS)
pub const SHFQA_SAMPLE_RATE_HZ: f64 = 2.0e9;
/// HDAWG sampling rate (2.4 GSPS)
pub const HDAWG_SAMPLE_RATE_HZ: f64 = 2.4e9;
/// Number of SHFQA channels
pub const SHFQA_CHANNELS: usize = 4;
/// Number of SHFSG channels
pub const SHFSG_CHANNELS: usize = 8;
/// Number of HDAWG channels
pub const HDAWG_CHANNELS: usize = 8;
/// Maximum RF frequency (Hz)
pub const MAX_RF_FREQ_HZ: f64 = 8.5e9;

// ============================================================================
// Enums
// ============================================================================

/// Zurich Instruments instrument type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ZIInstrumentType {
    SHFQA,
    SHFSG,
    HDAWG,
    PQSC,
    UHFQA,
}

/// Channel operation mode
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChannelMode {
    /// Spectroscopy mode (frequency sweep)
    Spectroscopy,
    /// Readout mode (I/Q demodulation)
    Readout,
    /// Drive mode (pulse generation)
    Drive,
}

/// Zurich Instruments driver errors
#[derive(Debug, Clone, PartialEq)]
pub enum ZIError {
    ConnectionFailed(String),
    InvalidChannel(usize),
    FrequencyOutOfRange(f64),
    AWGCompileError(String),
    TimeoutError,
    CalibrationError,
}

impl std::fmt::Display for ZIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZIError::ConnectionFailed(s) => write!(f, "Connection failed: {}", s),
            ZIError::InvalidChannel(c) => write!(f, "Invalid channel: {}", c),
            ZIError::FrequencyOutOfRange(freq) => write!(f, "Frequency out of range: {:.2e} Hz", freq),
            ZIError::AWGCompileError(msg) => write!(f, "AWG compile error: {}", msg),
            ZIError::TimeoutError => write!(f, "Timeout error"),
            ZIError::CalibrationError => write!(f, "Calibration error"),
        }
    }
}

// ============================================================================
// ZIPulse — pulse for SHFSG/HDAWG
// ============================================================================

/// Quantum pulse for Zurich Instruments devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZIPulse {
    pub id: String,
    /// Central oscillator frequency (Hz)
    pub center_frequency_hz: f64,
    /// Modulation frequency (Hz, offset from center)
    pub modulation_frequency_hz: f64,
    /// Normalized amplitude (0.0-1.0)
    pub amplitude: f64,
    /// Duration in ns
    pub duration_ns: u64,
    /// Phase in radians
    pub phase_rad: f64,
    /// Waveform I (normalized samples -1.0 a 1.0)
    pub waveform_i: Vec<f64>,
    /// Waveform Q (normalized samples -1.0 a 1.0)
    pub waveform_q: Vec<f64>,
    /// Output channel (0-indexed)
    pub channel: usize,
}

impl ZIPulse {
    /// Creates a Gaussian pulse
    pub fn gaussian(
        channel: usize,
        center_freq: f64,
        mod_freq: f64,
        amplitude: f64,
        sigma_ns: f64,
        duration_ns: u64,
        sample_rate: f64,
    ) -> Result<Self, ZIError> {
        if center_freq > MAX_RF_FREQ_HZ {
            return Err(ZIError::FrequencyOutOfRange(center_freq));
        }

        let n_samples = (duration_ns as f64 * sample_rate / 1e9) as usize;
        let sigma_samples = sigma_ns * sample_rate / 1e9;
        let center = n_samples as f64 / 2.0;

        let mut waveform_i = Vec::with_capacity(n_samples);
        let mut waveform_q = Vec::with_capacity(n_samples);

        for i in 0..n_samples {
            let t = i as f64;
            let t_ns = t / (sample_rate / 1e9);
            let gauss = (-(t - center).powi(2) / (2.0 * sigma_samples.powi(2))).exp();
            let phase = 2.0 * std::f64::consts::PI * mod_freq * t_ns * 1e-9;
            waveform_i.push(amplitude * gauss * phase.cos());
            waveform_q.push(amplitude * gauss * phase.sin());
        }

        Ok(Self {
            id: Uuid::new_v4().to_string(),
            center_frequency_hz: center_freq,
            modulation_frequency_hz: mod_freq,
            amplitude,
            duration_ns,
            phase_rad: 0.0,
            waveform_i,
            waveform_q,
            channel,
        })
    }

    /// Creates a DRAG pulse (Derivative Removal via Adiabatic Gate)
    pub fn drag(
        channel: usize,
        center_freq: f64,
        amplitude: f64,
        sigma_ns: f64,
        beta: f64,
        duration_ns: u64,
        sample_rate: f64,
    ) -> Result<Self, ZIError> {
        let mut base = Self::gaussian(channel, center_freq, 0.0, amplitude, sigma_ns, duration_ns, sample_rate)?;
        let n = base.waveform_i.len();
        let sigma_samples = sigma_ns * sample_rate / 1e9;
        let center = n as f64 / 2.0;

        // DRAG: Q component = -beta/sigma² × (t - center) × I(t)
        for i in 0..n {
            let t = i as f64 - center;
            let drag_q = -beta / sigma_samples.powi(2) * t * base.waveform_i[i];
            base.waveform_q[i] = drag_q;
        }

        Ok(base)
    }

    /// Creates π pulse (X gate)
    pub fn x_gate(qubit_freq_hz: f64, channel: usize) -> Result<Self, ZIError> {
        Self::drag(channel, qubit_freq_hz, 1.0, 8.0, 0.5, 40, SHFSG_CHANNELS as f64 * 1e8)
    }

    /// Validate pulse parameters
    pub fn validate(&self) -> Result<(), ZIError> {
        if self.center_frequency_hz > MAX_RF_FREQ_HZ {
            return Err(ZIError::FrequencyOutOfRange(self.center_frequency_hz));
        }
        if self.amplitude < 0.0 || self.amplitude > 1.0 {
            return Err(ZIError::AWGCompileError("Amplitude outside [0, 1]".into()));
        }
        Ok(())
    }

    /// Waveform length in samples
    pub fn num_samples(&self) -> usize {
        self.waveform_i.len()
    }
}

// ============================================================================
// SHFQAChannel — SHFQA readout channel
// ============================================================================

/// SHFQA channel for resonator readout
#[derive(Debug)]
pub struct SHFQAChannel {
    pub index: usize,
    pub center_frequency_hz: f64,
    pub mode: ChannelMode,
    pub integration_time_ns: u64,
    pub readout_weights: Vec<(Vec<f64>, Vec<f64>)>, // (wi, wq) por qubit
    pub measurement_results: Vec<f64>,
}

impl SHFQAChannel {
    pub fn new(index: usize, center_freq: f64) -> Self {
        Self {
            index,
            center_frequency_hz: center_freq,
            mode: ChannelMode::Readout,
            integration_time_ns: 2000,
            readout_weights: Vec::new(),
            measurement_results: Vec::new(),
        }
    }

    /// Configures integration kernel for a qubit
    pub fn set_integration_kernel(&mut self, weight_i: Vec<f64>, weight_q: Vec<f64>) {
        self.readout_weights.push((weight_i, weight_q));
    }

    /// Simulates readout of n qubits (returns |0⟩ probabilities)
    pub fn measure(&mut self, n_qubits: usize, fidelity: f64) -> Vec<f64> {
        use std::time::SystemTime;
        let seed = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos();

        self.measurement_results = (0..n_qubits)
            .map(|i| {
                // Deterministic simulation based on index and fidelity
                let raw = ((seed as usize + i * 7) % 1000) as f64 / 1000.0;
                fidelity * (1.0 - raw * 0.1)
            })
            .collect();

        self.measurement_results.clone()
    }
}

// ============================================================================
// ZIInstrument — generic instrument
// ============================================================================

/// Zurich Instruments instrument
#[derive(Debug)]
pub struct ZIInstrument {
    pub serial: String,
    pub instrument_type: ZIInstrumentType,
    pub ip_address: String,
    pub connected: bool,
    pub shfqa_channels: Vec<SHFQAChannel>,
    pub pulse_queue: Vec<ZIPulse>,
    pub pulses_executed: usize,
}

impl ZIInstrument {
    /// Creates an instrument in simulation mode
    pub fn new_simulated(instr_type: ZIInstrumentType, serial: impl Into<String>) -> Self {
        let n_channels = match instr_type {
            ZIInstrumentType::SHFQA => SHFQA_CHANNELS,
            ZIInstrumentType::SHFSG => SHFSG_CHANNELS,
            _ => 0,
        };

        let shfqa_channels: Vec<SHFQAChannel> = (0..n_channels)
            .map(|i| SHFQAChannel::new(i, 6.0e9 + i as f64 * 100e6))
            .collect();

        Self {
            serial: serial.into(),
            instrument_type: instr_type,
            ip_address: "192.168.1.200".into(),
            connected: true,
            shfqa_channels,
            pulse_queue: Vec::new(),
            pulses_executed: 0,
        }
    }

    /// Adds pulse to the AWG queue
    pub fn queue_pulse(&mut self, pulse: ZIPulse) -> Result<(), ZIError> {
        pulse.validate()?;
        self.pulse_queue.push(pulse);
        Ok(())
    }

    /// Compiles and runs the AWG queue (simulated)
    pub fn run_awg(&mut self) -> Result<usize, ZIError> {
        if !self.connected {
            return Err(ZIError::ConnectionFailed("Not connected".into()));
        }
        let n = self.pulse_queue.len();
        self.pulses_executed += n;
        self.pulse_queue.clear();
        Ok(n)
    }

    /// Qubit readout via SHFQA
    pub fn measure_qubit(&mut self, channel: usize, fidelity: f64) -> Result<f64, ZIError> {
        if channel >= self.shfqa_channels.len() {
            return Err(ZIError::InvalidChannel(channel));
        }
        let results = self.shfqa_channels[channel].measure(1, fidelity);
        Ok(results[0])
    }

    /// Overall instrument status
    pub fn status_summary(&self) -> String {
        format!(
            "{:?} [{}] — connected={}, queued={}, executed={}",
            self.instrument_type,
            self.serial,
            self.connected,
            self.pulse_queue.len(),
            self.pulses_executed
        )
    }
}

// ============================================================================
// ZurichSetup — complete rack (PQSC + SHFQA + SHFSG)
// ============================================================================

/// Complete Zurich Instruments setup for superconducting quantum computing
#[derive(Debug)]
pub struct ZurichSetup {
    pub setup_id: String,
    pub pqsc_serial: String,
    pub instruments: HashMap<String, ZIInstrument>,
}

impl ZurichSetup {
    pub fn new(pqsc_serial: impl Into<String>) -> Self {
        Self {
            setup_id: Uuid::new_v4().to_string(),
            pqsc_serial: pqsc_serial.into(),
            instruments: HashMap::new(),
        }
    }

    /// Typical setup for 17 qubits (1 SHFQA + 2 SHFSG)
    pub fn configure_17_qubit_setup() -> Self {
        let mut setup = Self::new("PQSC-001");
        let shfqa = ZIInstrument::new_simulated(ZIInstrumentType::SHFQA, "SHFQA-001");
        let shfsg1 = ZIInstrument::new_simulated(ZIInstrumentType::SHFSG, "SHFSG-001");
        let shfsg2 = ZIInstrument::new_simulated(ZIInstrumentType::SHFSG, "SHFSG-002");

        setup.instruments.insert("SHFQA-001".into(), shfqa);
        setup.instruments.insert("SHFSG-001".into(), shfsg1);
        setup.instruments.insert("SHFSG-002".into(), shfsg2);
        setup
    }

    /// Sends pulse to a specific instrument
    pub fn send_pulse(&mut self, serial: &str, pulse: ZIPulse) -> Result<(), ZIError> {
        let instr = self.instruments.get_mut(serial)
            .ok_or_else(|| ZIError::ConnectionFailed(format!("Instrument {} not found", serial)))?;
        instr.queue_pulse(pulse)
    }

    /// Triggers all instruments (synchronized through PQSC)
    pub fn trigger_all(&mut self) -> Result<usize, ZIError> {
        let mut total = 0;
        for instr in self.instruments.values_mut() {
            total += instr.run_awg()?;
        }
        Ok(total)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gaussian_pulse_creation() {
        let p = ZIPulse::gaussian(0, 5.0e9, 0.0, 1.0, 8.0, 40, SHFQA_SAMPLE_RATE_HZ);
        assert!(p.is_ok());
        let p = p.unwrap();
        assert!(!p.waveform_i.is_empty());
        assert_eq!(p.waveform_i.len(), p.waveform_q.len());
    }

    #[test]
    fn test_frequency_out_of_range() {
        let result = ZIPulse::gaussian(0, 9.5e9, 0.0, 1.0, 8.0, 40, SHFQA_SAMPLE_RATE_HZ);
        assert!(matches!(result, Err(ZIError::FrequencyOutOfRange(_))));
    }

    #[test]
    fn test_drag_pulse() {
        let p = ZIPulse::drag(0, 5.0e9, 0.8, 8.0, 0.5, 40, SHFQA_SAMPLE_RATE_HZ);
        assert!(p.is_ok());
    }

    #[test]
    fn test_instrument_queue_and_run() {
        let mut instr = ZIInstrument::new_simulated(ZIInstrumentType::SHFSG, "TEST-001");
        let pulse = ZIPulse::gaussian(0, 5.0e9, 0.0, 1.0, 8.0, 40, SHFQA_SAMPLE_RATE_HZ).unwrap();
        instr.queue_pulse(pulse).unwrap();
        let n = instr.run_awg().unwrap();
        assert_eq!(n, 1);
        assert_eq!(instr.pulses_executed, 1);
    }

    #[test]
    fn test_shfqa_measurement() {
        let mut ch = SHFQAChannel::new(0, 7.0e9);
        let results = ch.measure(3, 0.99);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|&r| r >= 0.0 && r <= 1.0));
    }

    #[test]
    fn test_17_qubit_setup() {
        let setup = ZurichSetup::configure_17_qubit_setup();
        assert_eq!(setup.instruments.len(), 3);
        assert!(setup.instruments.contains_key("SHFQA-001"));
    }

    #[test]
    fn test_trigger_all() {
        let mut setup = ZurichSetup::configure_17_qubit_setup();
        let pulse = ZIPulse::gaussian(0, 5.0e9, 0.0, 1.0, 8.0, 40, SHFQA_SAMPLE_RATE_HZ).unwrap();
        setup.send_pulse("SHFSG-001", pulse).unwrap();
        let total = setup.trigger_all().unwrap();
        assert_eq!(total, 1);
    }
}
