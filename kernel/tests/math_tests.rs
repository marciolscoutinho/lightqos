//! Tests for the math modules

use lightqos_kernel::math::geometric_algebra::*;
use lightqos_kernel::math::octave_algebra::*;

#[test]
fn test_ga3d_operations() {
    let v1 = GA3D::new(1.0, 0.0, 0.0);
    let v2 = GA3D::new(0.0, 1.0, 0.0);
    
    // Dot product
    assert_eq!(v1.dot(&v2), 0.0);
    
    // Cross product
    let cross = v1.cross(&v2);
    assert!((cross.x - 0.0).abs() < 1e-10);
    assert!((cross.y - 0.0).abs() < 1e-10);
    assert!((cross.z - 1.0).abs() < 1e-10);
    
    // Magnitude
    let v3 = GA3D::new(3.0, 4.0, 0.0);
    assert!((v3.magnitude() - 5.0).abs() < 1e-10);
}

#[test]
fn test_ga3d_normalization() {
    let v = GA3D::new(3.0, 4.0, 0.0);
    let normalized = v.normalize();
    
    assert!((normalized.magnitude() - 1.0).abs() < 1e-10);
    assert!((normalized.x - 0.6).abs() < 1e-10);
    assert!((normalized.y - 0.8).abs() < 1e-10);
}

#[test]
fn test_bivector_wedge() {
    let v1 = GA3D::new(1.0, 0.0, 0.0);
    let v2 = GA3D::new(0.0, 1.0, 0.0);
    
    let bivector = v1.wedge(&v2);
    assert!((bivector.magnitude() - 1.0).abs() < 1e-10);
}

#[test]
fn test_octave_positions() {
    assert_eq!(OctavePosition::Generation4Plus.as_index(), 4);
    assert_eq!(OctavePosition::Inertia0.as_index(), 0);
    assert_eq!(OctavePosition::Radiation4Minus.as_index(), -4);
}

#[test]
fn test_octave_cycle() {
    let mut pos = OctavePosition::Generation4Plus;
    let mut positions = vec![pos];
    
    while let Some(next) = pos.next() {
        positions.push(next);
        pos = next;
        if positions.len() > 10 {
            break;
        }
    }
    
    assert_eq!(positions.len(), 10); // 9 positions + restart
}

#[test]
fn test_harmonic_multipliers() {
    assert_eq!(OctavePosition::Generation4Plus.harmonic_multiplier(), 16.0);
    assert_eq!(OctavePosition::Generation2Plus.harmonic_multiplier(), 4.0);
    assert_eq!(OctavePosition::Inertia0.harmonic_multiplier(), 1.0);
    assert_eq!(OctavePosition::Radiation2Minus.harmonic_multiplier(), 0.25);
}

#[test]
fn test_octave_transition_energy() {
    let energy = transition_energy(
        OctavePosition::Inertia0,
        OctavePosition::Generation4Plus,
        100.0,
    );
    
    assert_eq!(energy, 400.0); // 4 octaves × 100
}

#[test]
fn test_octave_cycle_pressure() {
    let cycle = OctaveCycle::new();
    let profile = cycle.pressure_profile(1.0);
    
    assert_eq!(profile.len(), 9);
    
    // Checks the relationship P_exp ∝ d²
    for (pos, p_exp, _p_con) in &profile {
        let multiplier = pos.harmonic_multiplier();
        let expected_exp = multiplier * multiplier;
        assert!((p_exp - expected_exp).abs() < 1e-6);
    }
}
