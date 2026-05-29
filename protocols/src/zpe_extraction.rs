// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// zpe_extraction.rs — ZPE Extraction — Zero-Point Energy harvesting protocol
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 04-02-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use uuid::Uuid;
use std::collections::HashMap;

// ============================================================================
// ZPE EXTRACTOR
// ============================================================================

/// Zero-Point Energy Extractor
#[derive(Debug, Clone)]
pub struct ZPEExtractor {
    /// Unique ID
    pub id: Uuid,
    
    /// Active dimensions (3-37)
    pub dimensions: usize,
    
    /// Quantum vacuum state
    pub vacuum_state: VacuumEnergy,
    
    /// Resonant cavities
    pub cavities: Vec<ResonantCavity>,
    
    /// Accumulated extracted energy (J)
    pub total_energy_extracted: f64,
    
    /// Statistics
    pub stats: ExtractionStats,
}

impl ZPEExtractor {
    /// Creates a new extractor
    pub fn new(dimensions: usize) -> Self {
        assert!(dimensions >= 3 && dimensions <= 37, "Dimensions must be 3-37");
        
        Self {
            id: Uuid::new_v4(),
            dimensions,
            vacuum_state: VacuumEnergy::new(dimensions),
            cavities: Vec::new(),
            total_energy_extracted: 0.0,
            stats: ExtractionStats::default(),
        }
    }
    
    /// Adds a resonant cavity
    pub fn add_cavity(&mut self, frequency: f64, quality_factor: f64) -> Uuid {
        let cavity = ResonantCavity::new(frequency, quality_factor);
        let id = cavity.id;
        
        self.cavities.push(cavity);
        
        id
    }
    
    /// Extracts energy from the vacuum
    pub fn extract_energy(&mut self, duration: f64) -> ExtractionResult {
        // Zero-point energy per mode: E = (1/2) ℏ ω
        const HBAR: f64 = 1.054571817e-34; // ℏ (J·s)
        
        let mut total_extracted = 0.0;
        let mut extractions_per_cavity = HashMap::new();
        
        // For each resonant cavity
        for cavity in &mut self.cavities {
            // Energy available in the mode
            let mode_energy = 0.5 * HBAR * cavity.frequency;
            
            // Extraction efficiency based on the Q factor
            let efficiency = cavity.quality_factor / (1.0 + cavity.quality_factor);
            
            // Energy extracted from this cavity
            let extracted = mode_energy * efficiency * duration * 1.0e15; // Theoretical amplification factor
            
            cavity.energy_extracted += extracted;
            total_extracted += extracted;
            
            extractions_per_cavity.insert(cavity.id, extracted);
        }
        
        // Update accumulated value
        self.total_energy_extracted += total_extracted;
        
        // Update statistics
        self.stats.total_extractions += 1;
        self.stats.total_energy += total_extracted;
        self.stats.avg_power = self.stats.total_energy / (self.stats.total_extractions as f64 * duration);
        
        ExtractionResult {
            energy_extracted: total_extracted,
            duration,
            power: total_extracted / duration,
            num_cavities: self.cavities.len(),
            success: total_extracted > 0.0,
        }
    }
    
    /// Calculates the vacuum energy density
    pub fn vacuum_energy_density(&self) -> f64 {
        // Vacuum energy density (simplified)
        // ρ = Σ (1/2) ℏ ω / V
        
        const HBAR: f64 = 1.054571817e-34;
        let volume = 1.0e-9; // 1 nm³ (typical cavity volume)
        
        let mut density = 0.0;
        
        for cavity in &self.cavities {
            density += 0.5 * HBAR * cavity.frequency / volume;
        }
        
        // Dimensional factor (more dimensions = more energy)
        density *= self.dimensions as f64 / 3.0;
        
        density
    }
    
    /// Optimizes cavities for maximum extraction
    pub fn optimize_cavities(&mut self) {
        // Optimize frequencies for octave resonance
        for (i, cavity) in self.cavities.iter_mut().enumerate() {
            // Align with one of the 10 octaves
            let octave = (i % 10) + 1;
            cavity.frequency = 2.0_f64.powi((octave - 1) as i32);
            
            // Improve the Q factor
            cavity.quality_factor *= 1.1; // 10% improvement
        }
    }
}

// ============================================================================
// VACUUM ENERGY
// ============================================================================

/// Quantum vacuum state
#[derive(Debug, Clone)]
pub struct VacuumEnergy {
    /// Active dimensions
    pub dimensions: usize,
    
    /// Energy density (J/m³)
    pub energy_density: f64,
    
    /// Quantum fluctuations
    pub fluctuations: f64,
    
    /// Vacuum entropy
    pub entropy: f64,
}

impl VacuumEnergy {
    pub fn new(dimensions: usize) -> Self {
        // Energy density increases with the number of dimensions
        let base_density = 1.0e-9; // J/m³ (theoretical estimate)
        let energy_density = base_density * (dimensions as f64 / 3.0).powi(4);
        
        Self {
            dimensions,
            energy_density,
            fluctuations: 1.0e-10, // Typical fluctuations
            entropy: 0.0, // The vacuum has zero entropy (ground state)
        }
    }
    
    /// Total energy in a volume
    pub fn energy_in_volume(&self, volume: f64) -> f64 {
        self.energy_density * volume
    }
}

// ============================================================================
// RESONANT CAVITY
// ============================================================================

/// Resonant cavity for extraction
#[derive(Debug, Clone)]
pub struct ResonantCavity {
    /// Unique ID
    pub id: Uuid,
    
    /// Resonance frequency (Hz)
    pub frequency: f64,
    
    /// Quality factor Q
    pub quality_factor: f64,
    
    /// Energy extracted from this cavity (J)
    pub energy_extracted: f64,
    
    /// Cavity volume (m³)
    pub volume: f64,
}

impl ResonantCavity {
    pub fn new(frequency: f64, quality_factor: f64) -> Self {
        Self {
            id: Uuid::new_v4(),
            frequency,
            quality_factor,
            energy_extracted: 0.0,
            volume: 1.0e-27, // 1 nm³
        }
    }
    
    /// Extraction rate (W)
    pub fn extraction_rate(&self) -> f64 {
        const HBAR: f64 = 1.054571817e-34;
        
        // Rate proportional to Q and ω
        let rate = HBAR * self.frequency * self.quality_factor / 1.0e20;
        rate
    }
}

// ============================================================================
// EXTRACTION RESULT
// ============================================================================

/// Extraction result
#[derive(Debug, Clone)]
pub struct ExtractionResult {
    /// Extracted energy (J)
    pub energy_extracted: f64,
    
    /// Extraction duration (s)
    pub duration: f64,
    
    /// Average power (W)
    pub power: f64,
    
    /// Number of cavities used
    pub num_cavities: usize,
    
    /// Was the extraction successful?
    pub success: bool,
}

// ============================================================================
// STATISTICS
// ============================================================================

/// Extraction statistics
#[derive(Debug, Clone, Default)]
pub struct ExtractionStats {
    /// Total number of extractions
    pub total_extractions: usize,
    
    /// Total extracted energy (J)
    pub total_energy: f64,
    
    /// Average power (W)
    pub avg_power: f64,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_zpe_extractor() {
        let extractor = ZPEExtractor::new(18);
        assert_eq!(extractor.dimensions, 18);
    }
    
    #[test]
    fn test_add_cavity() {
        let mut extractor = ZPEExtractor::new(18);
        let id = extractor.add_cavity(1.0e14, 1000.0);
        
        assert_eq!(extractor.cavities.len(), 1);
        assert!(extractor.cavities.iter().any(|c| c.id == id));
    }
    
    #[test]
    fn test_extract_energy() {
        let mut extractor = ZPEExtractor::new(37); // Maximum dimensions
        
        // Add multiple cavities
        for i in 1..=10 {
            let freq = 2.0_f64.powi((i - 1) as i32); // Octaves
            extractor.add_cavity(freq, 1000.0);
        }
        
        let result = extractor.extract_energy(1.0);
        
        assert!(result.success);
        assert!(result.energy_extracted > 0.0);
    }
    
    #[test]
    fn test_vacuum_energy_density() {
        let mut extractor = ZPEExtractor::new(37);
        extractor.add_cavity(1.0e14, 1000.0);
        
        let density = extractor.vacuum_energy_density();
        assert!(density > 0.0);
    }
}
