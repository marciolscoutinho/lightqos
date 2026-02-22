//! Octave Algebra (Blocked Potentials Formula)
//! 4+ 3+ 2+ 1+ 0= 1- 2- 3- 4-

use std::fmt;

/// Position in the Blocked Potentials Formula
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OctavePosition {
    Generation4Plus,
    Generation3Plus,
    Generation2Plus,
    Generation1Plus,
    Inertia0,
    Radiation1Minus,
    Radiation2Minus,
    Radiation3Minus,
    Radiation4Minus,
}

impl OctavePosition {
    /// Returns the numeric index (-4 to +4)
    pub fn as_index(&self) -> i8 {
        match self {
            OctavePosition::Generation4Plus => 4,
            OctavePosition::Generation3Plus => 3,
            OctavePosition::Generation2Plus => 2,
            OctavePosition::Generation1Plus => 1,
            OctavePosition::Inertia0 => 0,
            OctavePosition::Radiation1Minus => -1,
            OctavePosition::Radiation2Minus => -2,
            OctavePosition::Radiation3Minus => -3,
            OctavePosition::Radiation4Minus => -4,
        }
    }
    
    /// Creates from an index
    pub fn from_index(index: i8) -> Option<Self> {
        match index {
            4 => Some(OctavePosition::Generation4Plus),
            3 => Some(OctavePosition::Generation3Plus),
            2 => Some(OctavePosition::Generation2Plus),
            1 => Some(OctavePosition::Generation1Plus),
            0 => Some(OctavePosition::Inertia0),
            -1 => Some(OctavePosition::Radiation1Minus),
            -2 => Some(OctavePosition::Radiation2Minus),
            -3 => Some(OctavePosition::Radiation3Minus),
            -4 => Some(OctavePosition::Radiation4Minus),
            _ => None,
        }
    }
    
    /// Harmonic multiplier (for temporal scheduling)
    /// Based on musical/physical frequency relationships
    pub fn harmonic_multiplier(&self) -> f64 {
        match self {
            OctavePosition::Generation4Plus => 16.0,   // 2^4
            OctavePosition::Generation3Plus => 8.0,    // 2^3
            OctavePosition::Generation2Plus => 4.0,    // 2^2
            OctavePosition::Generation1Plus => 2.0,    // 2^1
            OctavePosition::Inertia0 => 1.0,           // 2^0
            OctavePosition::Radiation1Minus => 0.5,    // 2^-1
            OctavePosition::Radiation2Minus => 0.25,   // 2^-2
            OctavePosition::Radiation3Minus => 0.125,  // 2^-3
            OctavePosition::Radiation4Minus => 0.0625, // 2^-4
        }
    }
    
    /// Returns whether it is in the Generation phase
    pub fn is_generation(&self) -> bool {
        self.as_index() > 0
    }
    
    /// Returns whether it is in the Radiation phase
    pub fn is_radiation(&self) -> bool {
        self.as_index() < 0
    }
    
    /// Returns whether it is at the Inertia point
    pub fn is_inertia(&self) -> bool {
        self.as_index() == 0
    }
    
    /// Transition to the next octave (full cycle)
    pub fn next(&self) -> Option<Self> {
        let current = self.as_index();
        if current == -4 {
            Some(OctavePosition::Generation4Plus) // Restarts the cycle
        } else {
            OctavePosition::from_index(current - 1)
        }
    }
    
    /// Transition to the previous octave
    pub fn previous(&self) -> Option<Self> {
        let current = self.as_index();
        if current == 4 {
            None // Start of the cycle
        } else {
            OctavePosition::from_index(current + 1)
        }
    }
}

impl fmt::Display for OctavePosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            OctavePosition::Generation4Plus => "4+",
            OctavePosition::Generation3Plus => "3+",
            OctavePosition::Generation2Plus => "2+",
            OctavePosition::Generation1Plus => "1+",
            OctavePosition::Inertia0 => "0=",
            OctavePosition::Radiation1Minus => "1-",
            OctavePosition::Radiation2Minus => "2-",
            OctavePosition::Radiation3Minus => "3-",
            OctavePosition::Radiation4Minus => "4-",
        })
    }
}

/// Full cycle of the 10 Octaves
pub struct OctaveCycle {
    positions: [OctavePosition; 9],
}

impl OctaveCycle {
    pub fn new() -> Self {
        OctaveCycle {
            positions: [
                OctavePosition::Generation4Plus,
                OctavePosition::Generation3Plus,
                OctavePosition::Generation2Plus,
                OctavePosition::Generation1Plus,
                OctavePosition::Inertia0,
                OctavePosition::Radiation1Minus,
                OctavePosition::Radiation2Minus,
                OctavePosition::Radiation3Minus,
                OctavePosition::Radiation4Minus,
            ],
        }
    }
    
    /// Returns all positions in order
    pub fn all_positions(&self) -> &[OctavePosition] {
        &self.positions
    }
    
    /// Computes the relative pressure at each octave
    /// P_exp ∝ d², P_con ∝ 1/d²
    pub fn pressure_profile(&self, base_distance: f64) -> Vec<(OctavePosition, f64, f64)> {
        self.positions.iter().map(|pos| {
            let multiplier = pos.harmonic_multiplier();
            let distance = base_distance * multiplier;
            
            let p_expansion = distance * distance;
            let p_contraction = 1.0 / (distance * distance + 1e-10);
            
            (*pos, p_expansion, p_contraction)
        }).collect()
    }
}

/// Operations between octaves (potential difference)
pub fn octave_difference(from: OctavePosition, to: OctavePosition) -> i8 {
    to.as_index() - from.as_index()
}

/// Energy required for a transition between octaves
/// E ∝ |Δoctave| (simplified)
pub fn transition_energy(from: OctavePosition, to: OctavePosition, base_energy: f64) -> f64 {
    let diff = octave_difference(from, to).abs() as f64;
    base_energy * diff
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_octave_cycle() {
        let mut pos = OctavePosition::Generation4Plus;
        let mut count = 0;
        
        while let Some(next_pos) = pos.next() {
            count += 1;
            pos = next_pos;
            
            if count > 10 {
                break; // Prevents an infinite loop
            }
        }
        
        assert_eq!(count, 9); // 9 transitions in the cycle
    }
    
    #[test]
    fn test_harmonic_multipliers() {
        assert_eq!(OctavePosition::Generation4Plus.harmonic_multiplier(), 16.0);
        assert_eq!(OctavePosition::Inertia0.harmonic_multiplier(), 1.0);
        assert_eq!(OctavePosition::Radiation4Minus.harmonic_multiplier(), 0.0625);
    }
    
    #[test]
    fn test_transition_energy() {
        let energy = transition_energy(
            OctavePosition::Inertia0,
            OctavePosition::Generation4Plus,
            100.0
        );
        
        assert_eq!(energy, 400.0); // 4 octaves × 100
    }
}
