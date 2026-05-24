// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// geometric_algebra.rs — Geometric Algebra — multivector operations and products
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 20-04-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use nalgebra::Vector3;
use std::ops::{Add, Sub, Mul, Neg};
use std::fmt;

// ============================================================================
// MULTIVECTOR
// ============================================================================

/// Multivetor em GA(3,0)
///
/// Um multivetor é a soma geral de elementos de diferentes graus:
/// M = α + a₁e₁ + a₂e₂ + a₃e₃ + b₁e₁₂ + b₂e₃₁ + b₃e₂₃ + βe₁₂₃
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Multivector {
    /// Parte escalar (grau 0)
    pub scalar: f64,
    
    /// Parte vetorial (grau 1): [e1, e2, e3]
    pub vector: [f64; 3],
    
    /// Parte bivetorial (grau 2): [e12, e31, e23]
    pub bivector: [f64; 3],
    
    /// Parte trivetorial/pseudoscalar (grau 3): e123
    pub pseudoscalar: f64,
}

impl Multivector {
    /// Creates multivetor zero
    pub fn zero() -> Self {
        Self {
            scalar: 0.0,
            vector: [0.0, 0.0, 0.0],
            bivector: [0.0, 0.0, 0.0],
            pseudoscalar: 0.0,
        }
    }
    
    /// Creates escalar puro
    pub fn scalar(s: f64) -> Self {
        Self {
            scalar: s,
            ..Self::zero()
        }
    }
    
    /// Creates vetor puro
    pub fn vector(x: f64, y: f64, z: f64) -> Self {
        Self {
            vector: [x, y, z],
            ..Self::zero()
        }
    }
    
    /// Creates vetor a partir de Vector3
    pub fn from_vector3(v: Vector3<f64>) -> Self {
        Self::vector(v.x, v.y, v.z)
    }
    
    /// Creates bivetor puro
    pub fn bivector(xy: f64, zx: f64, yz: f64) -> Self {
        Self {
            bivector: [xy, zx, yz],
            ..Self::zero()
        }
    }
    
    /// Creates pseudoscalar puro
    pub fn pseudoscalar(p: f64) -> Self {
        Self {
            pseudoscalar: p,
            ..Self::zero()
        }
    }
    
    /// Magnitude (norma) do multivetor
    pub fn magnitude(&self) -> f64 {
        (self.scalar * self.scalar
         + self.vector[0] * self.vector[0]
         + self.vector[1] * self.vector[1]
         + self.vector[2] * self.vector[2]
         + self.bivector[0] * self.bivector[0]
         + self.bivector[1] * self.bivector[1]
         + self.bivector[2] * self.bivector[2]
         + self.pseudoscalar * self.pseudoscalar).sqrt()
    }
    
    /// Normaliza multivetor
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
    
    /// Produto geométrico (fundamental em GA)
    pub fn geometric_product(&self, other: &Self) -> Self {
        // Implementação completa do produto geométrico
        // ab = a·b + a∧b (for vetores)
        
        // Componentes do resultado
        let mut result = Self::zero();
        
        // Escalar × tudo
        result.scalar += self.scalar * other.scalar;
        result.vector[0] += self.scalar * other.vector[0];
        result.vector[1] += self.scalar * other.vector[1];
        result.vector[2] += self.scalar * other.vector[2];
        result.bivector[0] += self.scalar * other.bivector[0];
        result.bivector[1] += self.scalar * other.bivector[1];
        result.bivector[2] += self.scalar * other.bivector[2];
        result.pseudoscalar += self.scalar * other.pseudoscalar;
        
        // Vetor × vetor
        // ei * ej = -ej * ei (anti-comutativo)
        // ei * ei = 1
        
        // Produto escalar (parte simétrica)
        let dot = self.vector[0] * other.vector[0]
                + self.vector[1] * other.vector[1]
                + self.vector[2] * other.vector[2];
        result.scalar += dot;
        
        // Produto exterior (parte anti-simétrica) → bivetor
        result.bivector[0] += self.vector[0] * other.vector[1] - self.vector[1] * other.vector[0]; // e12
        result.bivector[1] += self.vector[2] * other.vector[0] - self.vector[0] * other.vector[2]; // e31
        result.bivector[2] += self.vector[1] * other.vector[2] - self.vector[2] * other.vector[1]; // e23
        
        // Vetor × bivetor → vetor + pseudoscalar
        result.vector[0] += self.vector[1] * other.bivector[2] - self.vector[2] * other.bivector[1];
        result.vector[1] += self.vector[2] * other.bivector[0] - self.vector[0] * other.bivector[2];
        result.vector[2] += self.vector[0] * other.bivector[1] - self.vector[1] * other.bivector[0];
        
        result.pseudoscalar += self.vector[0] * other.bivector[0]
                              + self.vector[1] * other.bivector[2]
                              + self.vector[2] * other.bivector[1];
        
        // Bivetor × bivetor → escalar + bivetor
        result.scalar -= self.bivector[0] * other.bivector[0]
                       + self.bivector[1] * other.bivector[1]
                       + self.bivector[2] * other.bivector[2];
        
        // Pseudoscalar × vetor
        result.bivector[0] += self.pseudoscalar * other.vector[2];
        result.bivector[1] += -self.pseudoscalar * other.vector[1];
        result.bivector[2] += self.pseudoscalar * other.vector[0];
        
        result
    }
    
    /// Produto exterior (wedge product)
    pub fn wedge(&self, other: &Self) -> Self {
        let mut result = Self::zero();
        
        // Escalar ∧ qualquer coisa = escalar × qualquer coisa
        result.vector[0] = self.scalar * other.vector[0];
        result.vector[1] = self.scalar * other.vector[1];
        result.vector[2] = self.scalar * other.vector[2];
        
        // Vetor ∧ vetor = bivetor
        result.bivector[0] = self.vector[0] * other.vector[1] - self.vector[1] * other.vector[0];
        result.bivector[1] = self.vector[2] * other.vector[0] - self.vector[0] * other.vector[2];
        result.bivector[2] = self.vector[1] * other.vector[2] - self.vector[2] * other.vector[1];
        
        // Vetor ∧ bivetor = pseudoscalar
        result.pseudoscalar = self.vector[0] * other.bivector[0]
                            + self.vector[1] * other.bivector[2]
                            + self.vector[2] * other.bivector[1];
        
        result
    }
    
    /// Inner product (contração)
    pub fn dot(&self, other: &Self) -> f64 {
        self.vector[0] * other.vector[0]
            + self.vector[1] * other.vector[1]
            + self.vector[2] * other.vector[2]
    }
    
    /// Dual (Hodge star)
    pub fn dual(&self) -> Self {
        // ⋆v = v × I, onde I = e123
        Self {
            scalar: self.pseudoscalar,
            vector: [self.bivector[0], self.bivector[1], self.bivector[2]],
            bivector: [self.vector[0], self.vector[1], self.vector[2]],
            pseudoscalar: self.scalar,
        }
    }
    
    /// Reverso (inversão da ordem dos vetores)
    pub fn reverse(&self) -> Self {
        Self {
            scalar: self.scalar,
            vector: self.vector,
            bivector: [-self.bivector[0], -self.bivector[1], -self.bivector[2]],
            pseudoscalar: -self.pseudoscalar,
        }
    }
    
    /// Rotor (rotação)
    pub fn rotor(axis: Vector3<f64>, angle: f64) -> Self {
        let half_angle = angle / 2.0;
        let normalized_axis = axis.normalize();
        
        // R = cos(θ/2) - sin(θ/2) n̂
        // onde n̂ é o bivetor dual ao eixo
        Self {
            scalar: half_angle.cos(),
            vector: [0.0, 0.0, 0.0],
            bivector: [
                -half_angle.sin() * normalized_axis.x,
                -half_angle.sin() * normalized_axis.y,
                -half_angle.sin() * normalized_axis.z,
            ],
            pseudoscalar: 0.0,
        }
    }
    
    /// Applies rotação a um vetor
    pub fn rotate_vector(&self, v: Vector3<f64>) -> Vector3<f64> {
        let vec = Self::from_vector3(v);
        let rotated = self.geometric_product(&vec).geometric_product(&self.reverse());
        
        Vector3::new(
            rotated.vector[0],
            rotated.vector[1],
            rotated.vector[2],
        )
    }
}

// ============================================================================
// OPERADORES
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
        write!(f, "{:.3} + {:.3}e1 + {:.3}e2 + {:.3}e3 + {:.3}e12 + {:.3}e31 + {:.3}e23 + {:.3}e123",
               self.scalar,
               self.vector[0], self.vector[1], self.vector[2],
               self.bivector[0], self.bivector[1], self.bivector[2],
               self.pseudoscalar)
    }
}

// ============================================================================
// ALIASES
// ============================================================================

pub type GA3D = Multivector;
pub type Blade = Multivector;
pub type GeometricProduct = Multivector;

// ============================================================================
// TESTES
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
