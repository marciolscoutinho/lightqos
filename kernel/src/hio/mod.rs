//! HIO - Holographic I/O
//! Enriched quantum measurement interface

pub mod shadow_copy;
pub mod observable_view;
pub mod statistical_guarantee;

use std::collections::HashMap;

/// Complete Holographic I/O system
pub struct HolographicIO {
    /// Statistical precision configuration
    pub precision_config: PrecisionConfig,
    
    /// Cache of recent measurements
    measurement_cache: HashMap<String, HolographicMeasurement>,
}

impl HolographicIO {
    pub fn new() -> Self {
        HolographicIO {
            precision_config: PrecisionConfig::default(),
            measurement_cache: HashMap::new(),
        }
    }
    
    /// Performs a full holographic measurement
    pub fn measure_holographic(
        &mut self,
        qubits: &[String],
        config: MeasurementConfig,
    ) -> Result<HolographicMeasurement, HIOError> {
        // 1. Shadow Copies
        let shadows = shadow_copy::collect_shadows(qubits, config.num_shadows)?;
        
        // 2. Observable Views (multi-basis)
        let views = observable_view::measure_multi_base(qubits, &config.bases)?;
        
        // 3. Statistical Guarantees
        let guarantee = statistical_guarantee::compute_guarantee(
            &shadows,
            &views,
            &self.precision_config,
        )?;
        
        let measurement = HolographicMeasurement {
            qubits: qubits.to_vec(),
            shadows,
            views,
            guarantee,
        };
        
        // Cache for later analysis
        let cache_key = format!("meas_{}", qubits.join("_"));
        self.measurement_cache.insert(cache_key, measurement.clone());
        
        Ok(measurement)
    }
}

/// Holographic measurement configuration
pub struct MeasurementConfig {
    pub num_shadows: usize,
    pub bases: Vec<MeasurementBasis>,
    pub confidence_level: f64,
}

impl Default for MeasurementConfig {
    fn default() -> Self {
        MeasurementConfig {
            num_shadows: 1000,
            bases: vec![
                MeasurementBasis::Z,
                MeasurementBasis::X,
                MeasurementBasis::Y,
            ],
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MeasurementBasis {
    Z,  // Computational basis
    X,  // Hadamard basis
    Y,  // Y basis
}

/// Full result of a holographic measurement
#[derive(Clone)]
pub struct HolographicMeasurement {
    pub qubits: Vec<String>,
    pub shadows: shadow_copy::ShadowData,
    pub views: observable_view::MultiBaseViews,
    pub guarantee: statistical_guarantee::StatisticalGuarantee,
}

/// Statistical precision configuration
pub struct PrecisionConfig {
    pub min_samples: usize,
    pub max_error: f64,
    pub confidence_level: f64,
}

impl Default for PrecisionConfig {
    fn default() -> Self {
        PrecisionConfig {
            min_samples: 1000,
            max_error: 0.01,
            confidence_level: 0.95,
        }
    }
}

#[derive(Debug)]
pub enum HIOError {
    InsufficientSamples,
    ConfidenceNotMet,
    MeasurementFailed(String),
}
