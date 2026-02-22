//! 3D Geometric Algebra
//! Basis for the Ether’s spatial representation

use std::ops::{Add, Sub, Mul, Neg};

/// Vector in 3D geometric algebra (Euclidean space)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GA3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl GA3D {
    /// Creates a new 3D vector
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        GA3D { x, y, z }
    }
    
    /// Zero vector
    pub fn zero() -> Self {
        GA3D::new(0.0, 0.0, 0.0)
    }
    
    /// Basis vectors
    pub fn e1() -> Self { GA3D::new(1.0, 0.0, 0.0) }
    pub fn e2() -> Self { GA3D::new(0.0, 1.0, 0.0) }
    pub fn e3() -> Self { GA3D::new(0.0, 0.0, 1.0) }
    
    /// Magnitude (Euclidean norm)
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    /// Normalizes the vector (unit length)
    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag < 1e-10 {
            GA3D::zero()
        } else {
            GA3D::new(self.x / mag, self.y / mag, self.z / mag)
        }
    }
    
    /// Dot product
    pub fn dot(&self, other: &GA3D) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    
    /// Cross product
    pub fn cross(&self, other: &GA3D) -> GA3D {
        GA3D {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }
    
    /// Outer product (wedge product) — returns a bivector
    /// In 3D, equivalent to the cross product (as a dual vector)
    pub fn wedge(&self, other: &GA3D) -> Bivector3D {
        Bivector3D::from_cross(self.cross(other))
    }
    
    /// Distance between two points
    pub fn distance(&self, other: &GA3D) -> f64 {
        (*self - *other).magnitude()
    }
}

/// Operator implementations
impl Add for GA3D {
    type Output = GA3D;
    fn add(self, other: GA3D) -> GA3D {
        GA3D::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for GA3D {
    type Output = GA3D;
    fn sub(self, other: GA3D) -> GA3D {
        GA3D::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f64> for GA3D {
    type Output = GA3D;
    fn mul(self, scalar: f64) -> GA3D {
        GA3D::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Neg for GA3D {
    type Output = GA3D;
    fn neg(self) -> GA3D {
        GA3D::new(-self.x, -self.y, -self.z)
    }
}

/// Bivector (oriented plane) in 3D
#[derive(Debug, Clone, Copy)]
pub struct Bivector3D {
    pub xy: f64,  // Component e1 ∧ e2
    pub yz: f64,  // Component e2 ∧ e3
    pub zx: f64,  // Component e3 ∧ e1
}

impl Bivector3D {
    pub fn new(xy: f64, yz: f64, zx: f64) -> Self {
        Bivector3D { xy, yz, zx }
    }
    
    pub fn from_cross(v: GA3D) -> Self {
        // Hodge duality: vector ↔ bivector in 3D
        Bivector3D {
            xy: v.z,
            yz: v.x,
            zx: v.y,
        }
    }
    
    /// Bivector magnitude (area)
    pub fn magnitude(&self) -> f64 {
        (self.xy * self.xy + self.yz * self.yz + self.zx * self.zx).sqrt()
    }
}

/// Full multivector (scalar + vector + bivector + trivector)
#[derive(Debug, Clone)]
pub struct Multivector3D {
    pub scalar: f64,
    pub vector: GA3D,
    pub bivector: Bivector3D,
    pub trivector: f64,  // Pseudoscalar
}

impl Multivector3D {
    /// Full geometric product
    pub fn geometric_product(a: &Multivector3D, b: &Multivector3D) -> Multivector3D {
        // Full Clifford product implementation
        // (simplified — the full version requires complete expansion)
        
        let scalar = a.scalar * b.scalar 
                   + a.vector.dot(&b.vector)
                   - a.bivector.xy * b.bivector.xy
                   - a.bivector.yz * b.bivector.yz
                   - a.bivector.zx * b.bivector.zx
                   - a.trivector * b.trivector;
        
        let vector = b.vector * a.scalar 
                   + a.vector * b.scalar
                   + a.vector.cross(&b.vector);
        
        // Bivector and trivector (simplified)
        let bivector = Bivector3D::new(0.0, 0.0, 0.0); // Placeholder
        let trivector = 0.0; // Placeholder
        
        Multivector3D {
            scalar,
            vector,
            bivector,
            trivector,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ga3d_basic() {
        let v1 = GA3D::new(1.0, 0.0, 0.0);
        let v2 = GA3D::new(0.0, 1.0, 0.0);
        
        assert_eq!(v1.dot(&v2), 0.0);
        assert_eq!(v1.magnitude(), 1.0);
        
        let cross = v1.cross(&v2);
        assert!((cross.z - 1.0).abs() < 1e-10);
    }
    
    #[test]
    fn test_normalization() {
        let v = GA3D::new(3.0, 4.0, 0.0);
        let normalized = v.normalize();
        
        assert!((normalized.magnitude() - 1.0).abs() < 1e-10);
    }
}
