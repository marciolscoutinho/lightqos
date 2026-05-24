// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// lib.rs — Math crate — standalone advanced mathematics for LightQOS
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 18-07-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use std::f64::consts::PI;

// Submodules
pub mod geometric_algebra;
pub mod fiber_bundle;
pub mod rigged_hilbert;
pub mod octave_algebra;

// Re-exports
pub use geometric_algebra::{GA3D, Multivector, Blade, GeometricProduct};
pub use fiber_bundle::{FiberBundle, TangentBundle, CotangentBundle, Connection};
pub use rigged_hilbert::{RiggedHilbert, Distribution, TestFunction, DualSpace};
pub use octave_algebra::{OctaveAlgebra, OctaveSpace, HarmonicStructure};

// ============================================================================
// MATHEMATICAL CONSTANTS
// ============================================================================

/// Pi
pub const PI_CONST: f64 = PI;

/// e (Euler's number)
pub const E_CONST: f64 = std::f64::consts::E;

/// Golden ratio φ = (1 + √5)/2
pub const GOLDEN_RATIO: f64 = 1.618033988749895;

/// √2
pub const SQRT_2: f64 = std::f64::consts::SQRT_2;

/// Numerical precision
pub const EPSILON: f64 = 1.0e-12;

// ============================================================================
// MATHEMATICAL UTILITIES
// ============================================================================

/// Checks whether a number is approximately zero
pub fn is_zero(x: f64) -> bool {
    x.abs() < EPSILON
}

/// Checks whether two numbers are approximately equal
pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

/// Normalizes angle to interval [0, 2π)
pub fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle % (2.0 * PI);
    if a < 0.0 {
        a += 2.0 * PI;
    }
    a
}

/// Calculates factorial (up to 20)
pub fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        2 => 2,
        3 => 6,
        4 => 24,
        5 => 120,
        6 => 720,
        7 => 5040,
        8 => 40320,
        9 => 362880,
        10 => 3628800,
        _ => {
            if n > 20 {
                panic!("Factorial overflow for n > 20");
            }
            (1..=n).product()
        }
    }
}

/// Binomial coefficient C(n, k) = n! / (k! (n-k)!)
pub fn binomial(n: u64, k: u64) -> u64 {
    if k > n {
        return 0;
    }
    factorial(n) / (factorial(k) * factorial(n - k))
}

/// Sinc function: sinc(x) = sin(x)/x
pub fn sinc(x: f64) -> f64 {
    if is_zero(x) {
        1.0
    } else {
        x.sin() / x
    }
}

/// Bessel function J0 (order 0) - approximation
pub fn bessel_j0(x: f64) -> f64 {
    let ax = x.abs();
    
    if ax < 8.0 {
        // Taylor series
        let y = x * x;
        let ans1 = 57568490574.0 + y * (-13362590354.0 + y * (651619640.7
            + y * (-11214424.18 + y * (77392.33017 + y * (-184.9052456)))));
        let ans2 = 57568490411.0 + y * (1029532985.0 + y * (9494680.718
            + y * (59272.64853 + y * (267.8532712 + y))));
        ans1 / ans2
    } else {
        // Asymptotic approximation
        let z = 8.0 / ax;
        let y = z * z;
        let xx = ax - 0.785398164;
        let ans1 = 1.0 + y * (-0.1098628627e-2 + y * (0.2734510407e-4
            + y * (-0.2073370639e-5 + y * 0.2093887211e-6)));
        let ans2 = -0.1562499995e-1 + y * (0.1430488765e-3
            + y * (-0.6911147651e-5 + y * (0.7621095161e-6
                - y * 0.934935152e-7)));
        (2.0 / (PI * ax)).sqrt() * (xx.cos() * ans1 - z * xx.sin() * ans2)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_constants() {
        assert!((PI_CONST - std::f64::consts::PI).abs() < EPSILON);
        assert!((E_CONST - 2.718281828459045).abs() < EPSILON);
        assert!((GOLDEN_RATIO - 1.618033988749895).abs() < EPSILON);
    }
    
    #[test]
    fn test_is_zero() {
        assert!(is_zero(0.0));
        assert!(is_zero(1.0e-13));
        assert!(!is_zero(1.0e-10));
    }
    
    #[test]
    fn test_approx_equal() {
        assert!(approx_equal(1.0, 1.0 + 1.0e-13));
        assert!(!approx_equal(1.0, 1.001));
    }
    
    #[test]
    fn test_factorial() {
        assert_eq!(factorial(0), 1);
        assert_eq!(factorial(1), 1);
        assert_eq!(factorial(5), 120);
        assert_eq!(factorial(10), 3628800);
    }
    
    #[test]
    fn test_binomial() {
        assert_eq!(binomial(5, 2), 10);
        assert_eq!(binomial(10, 3), 120);
        assert_eq!(binomial(5, 0), 1);
        assert_eq!(binomial(5, 5), 1);
    }
    
    #[test]
    fn test_sinc() {
        assert!((sinc(0.0) - 1.0).abs() < EPSILON);
        assert!((sinc(PI) - 0.0).abs() < 1.0e-10);
    }
}

pub use octave_algebra::OctaveAlgebra;
pub use rigged_hilbert::{RiggedHilbertSpace, DiracState, NormType};
