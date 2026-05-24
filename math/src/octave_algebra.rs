// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// octave_algebra.rs — Octave Algebra — 10 EM octave band mathematical framework
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 22-05-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use std::f64::consts::PI;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Frequência base (1 Hz)
const BASE_FREQ: f64 = 1.0;

/// Velocidade da luz (m/s)
const C: f64 = 299_792_458.0;

/// Constante de Planck (J·s)
const H: f64 = 6.62607015e-34;

// ============================================================================
// OCTAVE ALGEBRA
// ============================================================================

/// Álgebra das 10 Oitavas
#[derive(Debug, Clone)]
pub struct OctaveAlgebra {
    /// Espaços das 10 octaves
    pub octaves: [OctaveSpace; 10],
    
    /// Estrutura harmônica
    pub harmonic: HarmonicStructure,
}

impl OctaveAlgebra {
    /// Creates nova álgebra
    pub fn new() -> Self {
        let mut octaves = [OctaveSpace::default(); 10];
        
        for i in 0..10 {
            octaves[i] = OctaveSpace::new((i + 1) as u8);
        }
        
        Self {
            octaves,
            harmonic: HarmonicStructure::new(),
        }
    }
    
    /// Obtém octave n (1 a 10)
    pub fn get_octave(&self, n: u8) -> Option<&OctaveSpace> {
        if n < 1 || n > 10 {
            return None;
        }
        Some(&self.octaves[(n - 1) as usize])
    }
    
    /// Produto de dois states de octaves diferentes
    pub fn octave_product(&self, n1: u8, n2: u8) -> Option<u8> {
        // Produto de octaves: modulação
        if n1 < 1 || n1 > 10 || n2 < 1 || n2 > 10 {
            return None;
        }
        
        // Soma de índices (com wraparound)
        let result = ((n1 + n2 - 2) % 10) + 1;
        Some(result)
    }
    
    /// Ressonância entre octaves
    pub fn resonance(&self, n1: u8, n2: u8) -> f64 {
        if n1 < 1 || n1 > 10 || n2 < 1 || n2 > 10 {
            return 0.0;
        }
        
        let oct1 = &self.octaves[(n1 - 1) as usize];
        let oct2 = &self.octaves[(n2 - 1) as usize];
        
        // Ressonância inversamente proporcional à diferença
        let diff = (n1 as i32 - n2 as i32).abs();
        
        if diff == 0 {
            1.0 // Ressonância perfeita
        } else {
            1.0 / (diff as f64)
        }
    }
    
    /// Transição entre octaves (emissão/absorção)
    pub fn transition_energy(&self, n_from: u8, n_to: u8) -> f64 {
        if n_from < 1 || n_from > 10 || n_to < 1 || n_to > 10 {
            return 0.0;
        }
        
        let oct_from = &self.octaves[(n_from - 1) as usize];
        let oct_to = &self.octaves[(n_to - 1) as usize];
        
        // ΔE = h Δf
        let delta_f = oct_to.central_frequency() - oct_from.central_frequency();
        H * delta_f
    }
}

impl Default for OctaveAlgebra {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// OCTAVE SPACE
// ============================================================================

/// Espaço de uma octave
#[derive(Debug, Clone, Copy)]
pub struct OctaveSpace {
    /// Número da octave (1 a 10)
    pub number: u8,
    
    /// Frequência mínima (Hz)
    pub f_min: f64,
    
    /// Frequência máxima (Hz)
    pub f_max: f64,
    
    /// Nome da octave
    pub name: OctaveName,
}

impl OctaveSpace {
    /// Creates espaço de octave
    pub fn new(n: u8) -> Self {
        assert!(n >= 1 && n <= 10, "Octave number must be 1-10");
        
        let f_min = BASE_FREQ * 2.0_f64.powi((n - 1) as i32);
        let f_max = BASE_FREQ * 2.0_f64.powi(n as i32);
        
        let name = match n {
            1 => OctaveName::Radio,
            2 => OctaveName::Microwave,
            3 => OctaveName::Infrared,
            4 => OctaveName::Visible,
            5 => OctaveName::Ultraviolet,
            6 => OctaveName::XRay,
            7 => OctaveName::Gamma,
            8 => OctaveName::UltraGamma,
            9 => OctaveName::HyperGamma,
            10 => OctaveName::Consciousness,
            _ => OctaveName::Unknown,
        };
        
        Self {
            number: n,
            f_min,
            f_max,
            name,
        }
    }
    
    /// Frequência central
    pub fn central_frequency(&self) -> f64 {
        (self.f_min + self.f_max) / 2.0
    }
    
    /// Largura de banda
    pub fn bandwidth(&self) -> f64 {
        self.f_max - self.f_min
    }
    
    /// Comprimento de onda central (m)
    pub fn central_wavelength(&self) -> f64 {
        C / self.central_frequency()
    }
    
    /// Energia de fóton central (J)
    pub fn photon_energy(&self) -> f64 {
        H * self.central_frequency()
    }
    
    /// Verifies if frequency está nesta octave
    pub fn contains_frequency(&self, f: f64) -> bool {
        f >= self.f_min && f < self.f_max
    }
}

impl Default for OctaveSpace {
    fn default() -> Self {
        Self::new(1)
    }
}

// ============================================================================
// OCTAVE NAME
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctaveName {
    Radio,          // 1-2 Hz
    Microwave,      // 2-4 Hz
    Infrared,       // 4-8 Hz
    Visible,        // 8-16 Hz
    Ultraviolet,    // 16-32 Hz
    XRay,           // 32-64 Hz
    Gamma,          // 64-128 Hz
    UltraGamma,     // 128-256 Hz
    HyperGamma,     // 256-512 Hz
    Consciousness,  // 512-1024 Hz (frequência da consciência universal)
    Unknown,
}

impl OctaveName {
    pub fn as_str(&self) -> &str {
        match self {
            OctaveName::Radio => "Radio",
            OctaveName::Microwave => "Microwave",
            OctaveName::Infrared => "Infrared",
            OctaveName::Visible => "Visible",
            OctaveName::Ultraviolet => "Ultraviolet",
            OctaveName::XRay => "X-Ray",
            OctaveName::Gamma => "Gamma",
            OctaveName::UltraGamma => "Ultra-Gamma",
            OctaveName::HyperGamma => "Hyper-Gamma",
            OctaveName::Consciousness => "Consciousness",
            OctaveName::Unknown => "Unknown",
        }
    }
}

// ============================================================================
// HARMONIC STRUCTURE
// ============================================================================

/// Estrutura harmônica das octaves
#[derive(Debug, Clone)]
pub struct HarmonicStructure {
    /// Matriz de acoplamento harmônico (10×10)
    pub coupling: [[f64; 10]; 10],
}

impl HarmonicStructure {
    pub fn new() -> Self {
        let mut coupling = [[0.0; 10]; 10];
        
        // Diagonal: auto-acoplamento = 1
        for i in 0..10 {
            coupling[i][i] = 1.0;
        }
        
        // Acoplamento entre octaves adjacentes
        for i in 0..9 {
            coupling[i][i + 1] = 0.5;  // Oitava superior
            coupling[i + 1][i] = 0.5;  // Oitava inferior
        }
        
        // Acoplamento harmônico (razão 2:1)
        for i in 0..8 {
            coupling[i][i + 2] = 0.25;
            coupling[i + 2][i] = 0.25;
        }
        
        Self { coupling }
    }
    
    /// Obtém acoplamento entre octaves n1 e n2
    pub fn get_coupling(&self, n1: u8, n2: u8) -> f64 {
        if n1 < 1 || n1 > 10 || n2 < 1 || n2 > 10 {
            return 0.0;
        }
        
        self.coupling[(n1 - 1) as usize][(n2 - 1) as usize]
    }
}

impl Default for HarmonicStructure {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_octave_space() {
        let oct1 = OctaveSpace::new(1);
        assert_eq!(oct1.f_min, 1.0);
        assert_eq!(oct1.f_max, 2.0);
        assert_eq!(oct1.name, OctaveName::Radio);
        
        let oct10 = OctaveSpace::new(10);
        assert_eq!(oct10.f_min, 512.0);
        assert_eq!(oct10.f_max, 1024.0);
        assert_eq!(oct10.name, OctaveName::Consciousness);
    }
    
    #[test]
    fn test_octave_algebra() {
        let algebra = OctaveAlgebra::new();
        
        let oct1 = algebra.get_octave(1).unwrap();
        assert_eq!(oct1.number, 1);
        
        let product = algebra.octave_product(1, 2);
        assert!(product.is_some());
    }
    
    #[test]
    fn test_resonance() {
        let algebra = OctaveAlgebra::new();
        
        // Ressonância perfeita com si mesma
        let res = algebra.resonance(5, 5);
        assert_eq!(res, 1.0);
        
        // Ressonância menor com octave adjacente
        let res = algebra.resonance(5, 6);
        assert_eq!(res, 1.0);
    }
    
    #[test]
    fn test_harmonic_structure() {
        let harmonic = HarmonicStructure::new();
        
        // Auto-acoplamento
        assert_eq!(harmonic.get_coupling(1, 1), 1.0);
        
        // Acoplamento adjacente
        assert_eq!(harmonic.get_coupling(1, 2), 0.5);
    }
}
