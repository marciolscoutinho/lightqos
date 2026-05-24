// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// field_driver.rs — EFAL Field Driver — hardware field interface and control
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 29-08-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::VecDeque;
use uuid::Uuid;

// ============================================================================
// FIELD DRIVER
// ============================================================================

/// Ether field driver
#[derive(Debug, Clone)]
pub struct FieldDriver {
    /// Control signal queue
    pub signal_queue: VecDeque<ControlSignal>,
    
    /// Current active signal
    pub active_signal: Option<ControlSignal>,
    
    /// History of executed signals
    pub signal_history: Vec<ControlSignal>,
    
    /// Is the driver enabled?
    pub enabled: bool,
    
    /// Output power (Watts)
    pub output_power: f64,
    
    /// Operating frequency (Hz)
    pub operating_frequency: f64,
}

impl FieldDriver {
    /// Creates a new driver
    pub fn new() -> Self {
        Self {
            signal_queue: VecDeque::new(),
            active_signal: None,
            signal_history: Vec::new(),
            enabled: true,
            output_power: 0.0,
            operating_frequency: 1.0e9, // 1 GHz default
        }
    }
    
    /// Enqueues a control signal
    pub fn enqueue_signal(&mut self, signal: ControlSignal) {
        self.signal_queue.push_back(signal);
    }
    
    /// Executes the next signal in the queue
    pub fn execute_next_signal(&mut self) -> Option<ControlSignal> {
        if let Some(signal) = self.signal_queue.pop_front() {
            self.active_signal = Some(signal.clone());
            self.signal_history.push(signal.clone());
            
            // Apply signal
            self.apply_signal(&signal);
            
            Some(signal)
        } else {
            self.active_signal = None;
            None
        }
    }
    
    /// Applies a signal to the field
    fn apply_signal(&mut self, signal: &ControlSignal) {
        match signal.signal_type {
            SignalType::Pulse(amplitude, _duration) => {
                self.output_power = amplitude;
            },
            SignalType::Continuous(amplitude) => {
                self.output_power = amplitude;
            },
            SignalType::Modulated(carrier_freq, _mod_freq) => {
                self.operating_frequency = carrier_freq;
            },
            SignalType::Shaped(ref shape) => {
                self.output_power = shape.amplitude;
            },
        }
    }
    
    /// Stops execution
    pub fn stop(&mut self) {
        self.enabled = false;
        self.output_power = 0.0;
        self.active_signal = None;
    }
    
    /// Resumes execution
    pub fn resume(&mut self) {
        self.enabled = true;
    }
    
    /// Clears the signal queue
    pub fn clear_queue(&mut self) {
        self.signal_queue.clear();
    }
    
    /// Number of signals in the queue
    pub fn queue_length(&self) -> usize {
        self.signal_queue.len()
    }
}

impl Default for FieldDriver {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CONTROL SIGNAL
// ============================================================================

/// Field control signal
#[derive(Debug, Clone)]
pub struct ControlSignal {
    /// Unique identifier
    pub id: Uuid,
    
    /// Signal type
    pub signal_type: SignalType,
    
    /// Priority (0-10)
    pub priority: u8,
    
    /// Timestamp
    pub timestamp: u64,
}

impl ControlSignal {
    /// Creates a new signal
    pub fn new(signal_type: SignalType, priority: u8) -> Self {
        Self {
            id: Uuid::new_v4(),
            signal_type,
            priority,
            timestamp: 0,
        }
    }
    
    /// Creates a pulse signal
    pub fn pulse(amplitude: f64, duration: f64) -> Self {
        Self::new(SignalType::Pulse(amplitude, duration), 5)
    }
    
    /// Creates a continuous signal
    pub fn continuous(amplitude: f64) -> Self {
        Self::new(SignalType::Continuous(amplitude), 3)
    }
}

// ============================================================================
// SIGNAL TYPE
// ============================================================================

/// Signal type
#[derive(Debug, Clone)]
pub enum SignalType {
    Pulse(f64, f64),
    Continuous(f64),
    Modulated(f64, f64),
    Shaped(SignalShape),
}

// ============================================================================
// SIGNAL SHAPE
// ============================================================================

/// Signal shape
#[derive(Debug, Clone)]
pub struct SignalShape {
    pub amplitude: f64,
    pub points: Vec<(f64, f64)>,
}

impl SignalShape {
    /// Creates a Gaussian shape
    pub fn gaussian(amplitude: f64, width: f64, num_points: usize) -> Self {
        let mut points = Vec::with_capacity(num_points);
        
        for i in 0..num_points {
            let t = -3.0 * width + 6.0 * width * (i as f64) / (num_points as f64);
            let a = amplitude * (-t * t / (2.0 * width * width)).exp();
            points.push((t, a));
        }
        
        Self { amplitude, points }
    }
}
