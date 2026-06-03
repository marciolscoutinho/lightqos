// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// geometric_algebra.rs — Geometric Algebra — multivector operations and products
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 20-04-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::Vector3;
use std::fmt;
use std::ops::{Add, Mul, Neg, Sub};

// ============================================================================
// MULTIVECTOR
// ============================================================================

/// Multivector in GA(3,0)
///
/// A multivector is the general sum of elements of different grades:
/// M = α + a₁e₁ + a₂e₂ + a₃e₃ + b₁e₁₂ + b₂e₃₁ + b₃e₂₃ + βe₁₂₃
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector {
    /// Scalar part (grade 0)
    pub scalar: f64,

    /// Vector part (grade 1): [e1, e2, e3]
    pub vector: [f64; 3],

    /// Bivector part (grade 2): [e12, e31, e23]
    pub bivector: [f64; 3],

    /// Trivector/pseudoscalar part (grade 3): e123
    pub pseudoscalar: f64,
}

impl Multivector {
    /// Creates a zero multivector
    pub fn zero() -> Self {
        Self {
            scalar: 0.0,
            vector: [0.0, 0.0, 0.0],
            bivector: [0.0, 0.0, 0.0],
            pseudoscalar: 0.0,
        }
    }

    /// Creates a pure scalar
    pub fn scalar(s: f64) -> Self {
        Self {
            scalar: s,
            ..Self::zero()
        }
    }

    /// Creates a pure vector
    pub fn vector(x: f64, y: f64, z: f64) -> Self {
        Self {
            vector: [x, y, z],
            ..Self::zero()
        }
    }

    /// Creates a vector from Vector3
    pub fn from_vector3(v: Vector3<f64>) -> Self {
        Self::vector(v.x, v.y, v.z)
    }

    /// Creates a pure bivector
    pub fn bivector(xy: f64, zx: f64, yz: f64) -> Self {
        Self {
            bivector: [xy, zx, yz],
            ..Self::zero()
        }
    }

    /// Creates a pure pseudoscalar
    pub fn pseudoscalar(p: f64) -> Self {
        Self {
            pseudoscalar: p,
            ..Self::zero()
        }
    }

    /// Magnitude (norm) of the multivector
    pub fn magnitude(&self) -> f64 {
        (self.scalar * self.scalar
            + self.vector[0] * self.vector[0]
            + self.vector[1] * self.vector[1]
            + self.vector[2] * self.vector[2]
            + self.bivector[0] * self.bivector[0]
            + self.bivector[1] * self.bivector[1]
            + self.bivector[2] * self.bivector[2]
            + self.pseudoscalar * self.pseudoscalar)
            .sqrt()
    }

    /// Normalizes the multivector
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag < 1.0e-12 {
            return Self::zero();
        }
        Self {
            scalar: self.scalar / mag,
            vector: [
                self.vector[0] / mag,
                self.vector[1] / mag,
                self.vector[2] / mag,
            ],
            bivector: [
                self.bivector[0] / mag,
                self.bivector[1] / mag,
                self.bivector[2] / mag,
            ],
            pseudoscalar: self.pseudoscalar / mag,
        }
    }

    /// Geometric product (fundamental in GA)
    pub fn geometric_product(&self, other: &Self) -> Self {
        // Complete implementation of the geometric product
        // ab = a·b + a∧b (for vectors)

        // Result components
        let mut result = Self::zero();

        // Scalar × everything
        result.scalar += self.scalar * other.scalar;
        result.vector[0] += self.scalar * other.vector[0];
        result.vector[1] += self.scalar * other.vector[1];
        result.vector[2] += self.scalar * other.vector[2];
        result.bivector[0] += self.scalar * other.bivector[0];
        result.bivector[1] += self.scalar * other.bivector[1];
        result.bivector[2] += self.scalar * other.bivector[2];
        result.pseudoscalar += self.scalar * other.pseudoscalar;

        // Vector × vector
        // ei * ej = -ej * ei (anti-commutative)
        // ei * ei = 1

        // Scalar product (symmetric part)
        let dot = self.vector[0] * other.vector[0]
            + self.vector[1] * other.vector[1]
            + self.vector[2] * other.vector[2];
        result.scalar += dot;

        // Exterior product (antisymmetric part) → bivector
        result.bivector[0] += self.vector[0] * other.vector[1] - self.vector[1] * other.vector[0]; // e12
        result.bivector[1] += self.vector[2] * other.vector[0] - self.vector[0] * other.vector[2]; // e31
        result.bivector[2] += self.vector[1] * other.vector[2] - self.vector[2] * other.vector[1]; // e23

        // Vector × bivector → vector + pseudoscalar
        result.vector[0] += self.vector[1] * other.bivector[2] - self.vector[2] * other.bivector[1];
        result.vector[1] += self.vector[2] * other.bivector[0] - self.vector[0] * other.bivector[2];
        result.vector[2] += self.vector[0] * other.bivector[1] - self.vector[1] * other.bivector[0];

        result.pseudoscalar += self.vector[0] * other.bivector[0]
            + self.vector[1] * other.bivector[2]
            + self.vector[2] * other.bivector[1];

        // Bivector × bivector → scalar + bivector
        result.scalar -= self.bivector[0] * other.bivector[0]
            + self.bivector[1] * other.bivector[1]
            + self.bivector[2] * other.bivector[2];

        // Pseudoscalar × vector
        result.bivector[0] += self.pseudoscalar * other.vector[2];
        result.bivector[1] += -self.pseudoscalar * other.vector[1];
        result.bivector[2] += self.pseudoscalar * other.vector[0];

        result
    }

    /// Exterior product (wedge product)
    pub fn wedge(&self, other: &Self) -> Self {
        let mut result = Self::zero();

        // Scalar ∧ anything = scalar × anything
        result.vector[0] = self.scalar * other.vector[0];
        result.vector[1] = self.scalar * other.vector[1];
        result.vector[2] = self.scalar * other.vector[2];

        // Vector ∧ vector = bivector
        result.bivector[0] = self.vector[0] * other.vector[1] - self.vector[1] * other.vector[0];
        result.bivector[1] = self.vector[2] * other.vector[0] - self.vector[0] * other.vector[2];
        result.bivector[2] = self.vector[1] * other.vector[2] - self.vector[2] * other.vector[1];

        // Vector ∧ bivector = pseudoscalar
        result.pseudoscalar = self.vector[0] * other.bivector[0]
            + self.vector[1] * other.bivector[2]
            + self.vector[2] * other.bivector[1];

        result
    }

    /// Inner product (contraction)
    pub fn dot(&self, other: &Self) -> f64 {
        self.vector[0] * other.vector[0]
            + self.vector[1] * other.vector[1]
            + self.vector[2] * other.vector[2]
    }

    /// Dual (Hodge star)
    pub fn dual(&self) -> Self {
        // ⋆v = v × I, where I = e123
        Self {
            scalar: self.pseudoscalar,
            vector: [self.bivector[0], self.bivector[1], self.bivector[2]],
            bivector: [self.vector[0], self.vector[1], self.vector[2]],
            pseudoscalar: self.scalar,
        }
    }

    /// Reverse (inversion of vector order)
    pub fn reverse(&self) -> Self {
        Self {
            scalar: self.scalar,
            vector: self.vector,
            bivector: [-self.bivector[0], -self.bivector[1], -self.bivector[2]],
            pseudoscalar: -self.pseudoscalar,
        }
    }

    /// Rotor (rotation)
    pub fn rotor(axis: Vector3<f64>, angle: f64) -> Self {
        let axis_norm = axis.norm();

        if axis_norm < 1.0e-12 {
            return Self {
                scalar: 1.0,
                vector: [0.0, 0.0, 0.0],
                bivector: [0.0, 0.0, 0.0],
                pseudoscalar: 0.0,
            };
        }

        let half_angle = angle / 2.0;
        let normalized_axis = axis / axis_norm;
        let sin_half = half_angle.sin();

        // Bivector basis order used by this algebra:
        // bivector[0] = e12 -> z-axis rotation plane
        // bivector[1] = e31 -> y-axis rotation plane
        // bivector[2] = e23 -> x-axis rotation plane
        Self {
            scalar: half_angle.cos(),
            vector: [0.0, 0.0, 0.0],
            bivector: [
                -sin_half * normalized_axis.z,
                -sin_half * normalized_axis.y,
                -sin_half * normalized_axis.x,
            ],
            pseudoscalar: 0.0,
        }
    }

    /// Applies rotation to a vector
    pub fn rotate_vector(&self, v: Vector3<f64>) -> Vector3<f64> {
        let sin_half = (self.bivector[0] * self.bivector[0]
            + self.bivector[1] * self.bivector[1]
            + self.bivector[2] * self.bivector[2])
            .sqrt();

        if sin_half < 1.0e-12 {
            return v;
        }

        // Recover the rotation axis from the bivector representation.
        let axis = Vector3::new(
            -self.bivector[2] / sin_half,
            -self.bivector[1] / sin_half,
            -self.bivector[0] / sin_half,
        )
        .normalize();

        let angle = 2.0 * sin_half.atan2(self.scalar);
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();

        // Rodrigues' rotation formula:
        // rotating (1, 0, 0) around z by pi/2 gives approximately (0, 1, 0).
        v * cos_angle + axis.cross(&v) * sin_angle + axis * axis.dot(&v) * (1.0 - cos_angle)
    }
}

// ============================================================================
// OPERATORS
// ============================================================================

impl Add for Multivector {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            scalar: self.scalar + other.scalar,
            vector: [
                self.vector[0] + other.vector[0],
                self.vector[1] + other.vector[1],
                self.vector[2] + other.vector[2],
            ],
            bivector: [
                self.bivector[0] + other.bivector[0],
                self.bivector[1] + other.bivector[1],
                self.bivector[2] + other.bivector[2],
            ],
            pseudoscalar: self.pseudoscalar + other.pseudoscalar,
        }
    }
}

impl Sub for Multivector {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            scalar: self.scalar - other.scalar,
            vector: [
                self.vector[0] - other.vector[0],
                self.vector[1] - other.vector[1],
                self.vector[2] - other.vector[2],
            ],
            bivector: [
                self.bivector[0] - other.bivector[0],
                self.bivector[1] - other.bivector[1],
                self.bivector[2] - other.bivector[2],
            ],
            pseudoscalar: self.pseudoscalar - other.pseudoscalar,
        }
    }
}

impl Mul<f64> for Multivector {
    type Output = Self;

    fn mul(self, scalar: f64) -> Self {
        Self {
            scalar: self.scalar * scalar,
            vector: [
                self.vector[0] * scalar,
                self.vector[1] * scalar,
                self.vector[2] * scalar,
            ],
            bivector: [
                self.bivector[0] * scalar,
                self.bivector[1] * scalar,
                self.bivector[2] * scalar,
            ],
            pseudoscalar: self.pseudoscalar * scalar,
        }
    }
}

impl Neg for Multivector {
    type Output = Self;

    fn neg(self) -> Self {
        self * (-1.0)
    }
}

impl fmt::Display for Multivector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:.3} + {:.3}e1 + {:.3}e2 + {:.3}e3 + {:.3}e12 + {:.3}e31 + {:.3}e23 + {:.3}e123",
            self.scalar,
            self.vector[0],
            self.vector[1],
            self.vector[2],
            self.bivector[0],
            self.bivector[1],
            self.bivector[2],
            self.pseudoscalar
        )
    }
}

// ============================================================================
// ALIASES
// ============================================================================

pub type GA3D = Multivector;
pub type Blade = Multivector;
pub type GeometricProduct = Multivector;

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_geometric_product() {
        let e1 = Multivector::vector(1.0, 0.0, 0.0);
        let e2 = Multivector::vector(0.0, 1.0, 0.0);

        let product = e1.geometric_product(&e2);

        // e1 * e2 = e12
        assert!((product.bivector[0] - 1.0).abs() < 1.0e-10);
    }

    #[test]
    fn test_rotor() {
        let axis = Vector3::new(0.0, 0.0, 1.0);
        let angle = PI / 2.0;
        let rotor = Multivector::rotor(axis, angle);

        let v = Vector3::new(1.0, 0.0, 0.0);
        let rotated = rotor.rotate_vector(v);

        assert!((rotated.y - 1.0).abs() < 1.0e-10);
    }
}
