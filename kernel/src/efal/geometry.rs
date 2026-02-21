//! Motion Cube Geometry
//! Based on TUCU: 3 inertial planes divide space into 8 compartments

use crate::math::geometric_algebra::GA3D;

/// Represents the fundamental geometry of quantum space
pub struct CubeGeometry {
    /// South Inertial Plane (0 = - equilibrium point)
    pub south_plane: Plane,
    
    /// North–South Vertical Plane (divides polarities)
    pub north_south_plane: Plane,
    
    /// East–West Vertical Plane (divides the same polarity)
    pub east_west_plane: Plane,
    
    /// 8 resulting cubic compartments
    pub compartments: [Compartment; 8],
}

impl CubeGeometry {
    /// Creates the default geometry based on the hardware dimensions
    pub fn from_hardware(config: &super::HardwareConfig) -> Self {
        // Inertial plane positions based on the physical topology
        let south_plane = Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0);
        let north_south_plane = Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0);
        let east_west_plane = Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0);
        
        // Computes the 8 compartments
        let compartments = Self::compute_compartments(
            &south_plane,
            &north_south_plane,
            &east_west_plane,
        );
        
        CubeGeometry {
            south_plane,
            north_south_plane,
            east_west_plane,
            compartments,
        }
    }
    
    /// Checks whether a position lies on an inertial plane (invalid for defects)
    pub fn is_valid_position(&self, position: &GA3D) -> bool {
        let tolerance = 1e-6;
        
        // Defects cannot be exactly on the inertial planes
        !(self.south_plane.contains(position, tolerance) ||
          self.north_south_plane.contains(position, tolerance) ||
          self.east_west_plane.contains(position, tolerance))
    }
    
    /// Identifies which compartment the position belongs to
    pub fn get_compartment(&self, position: &GA3D) -> Option<usize> {
        for (idx, compartment) in self.compartments.iter().enumerate() {
            if compartment.contains(position) {
                return Some(idx);
            }
        }
        None
    }
    
    fn compute_compartments(
        south: &Plane,
        north_south: &Plane,
        east_west: &Plane,
    ) -> [Compartment; 8] {
        // Implementation of the division into 8 compartments
        // Based on the intersection of the 3 planes
        [
            Compartment::new(0, vec![/* boundaries */]),
            Compartment::new(1, vec![]),
            Compartment::new(2, vec![]),
            Compartment::new(3, vec![]),
            Compartment::new(4, vec![]),
            Compartment::new(5, vec![]),
            Compartment::new(6, vec![]),
            Compartment::new(7, vec![]),
        ]
    }
    
    pub fn space_time_base(&self) -> SpaceTimeBase {
        SpaceTimeBase {
            spatial_dims: 3,
            temporal_dim: 1,
            geometry: self.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Plane {
    normal: GA3D,
    offset: f64,
}

impl Plane {
    pub fn new(normal: GA3D, offset: f64) -> Self {
        Plane { normal: normal.normalize(), offset }
    }
    
    pub fn contains(&self, point: &GA3D, tolerance: f64) -> bool {
        let distance = self.normal.dot(point) - self.offset;
        distance.abs() < tolerance
    }
}

pub struct Compartment {
    id: usize,
    boundaries: Vec<Plane>,
}

impl Compartment {
    pub fn new(id: usize, boundaries: Vec<Plane>) -> Self {
        Compartment { id, boundaries }
    }
    
    pub fn contains(&self, position: &GA3D) -> bool {
        self.boundaries.iter().all(|plane| {
            plane.normal.dot(position) - plane.offset > 0.0
        })
    }
}

pub struct SpaceTimeBase {
    pub spatial_dims: usize,
    pub temporal_dim: usize,
    pub geometry: CubeGeometry,
}
