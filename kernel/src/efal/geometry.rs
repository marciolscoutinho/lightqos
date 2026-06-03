// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// geometry.rs — EFAL Geometry — 37-dimensional Ether Field and 10 EM octaves
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 11-10-2023
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::math::geometric_algebra::GA3D;

/// Represents the fundamental geometry of quantum space
#[derive(Debug, Clone)]
pub struct CubeGeometry {
    /// South Inertial Plane (0= - equilibrium point)
    pub south_plane: Plane,

    /// North-South Vertical Plane (divides polarities)
    pub north_south_plane: Plane,

    /// East-West Vertical Plane (divides the same polarity)
    pub east_west_plane: Plane,

    /// 8 resulting cubic compartments
    pub compartments: [Compartment; 8],
}

impl CubeGeometry {
    /// Creates the standard geometry based on the hardware dimensions
    pub fn from_hardware(config: &super::HardwareConfig) -> Self {
        // Positions of the inertial planes based on the physical topology
        let south_plane = Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0);
        let north_south_plane = Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0);
        let east_west_plane = Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0);

        // Calculates the 8 compartments
        let compartments =
            Self::compute_compartments(&south_plane, &north_south_plane, &east_west_plane);

        CubeGeometry {
            south_plane,
            north_south_plane,
            east_west_plane,
            compartments,
        }
    }

    /// Checks whether a position lies on an inertial plane
    /// invalid for defects
    pub fn is_valid_position(&self, position: &GA3D) -> bool {
        let tolerance = 1e-6;

        // Defects cannot lie exactly on the inertial planes
        !(self.south_plane.contains(position, tolerance)
            || self.north_south_plane.contains(position, tolerance)
            || self.east_west_plane.contains(position, tolerance))
    }

    /// Identifies which compartment contains the given position
    pub fn get_compartment(&self, position: &GA3D) -> Option<usize> {
        for (idx, compartment) in self.compartments.iter().enumerate() {
            if compartment.contains(position) {
                return Some(idx);
            }
        }
        None
    }

    fn compute_compartments(
        _south: &Plane,
        _north_south: &Plane,
        _east_west: &Plane,
    ) -> [Compartment; 8] {
        // The three inertial planes divide the cube into 8 octants.
        //
        // Compartment id encoding:
        // bit 0 -> x < 0
        // bit 1 -> y < 0
        // bit 2 -> z < 0
        //
        // Therefore:
        // 0 = +x +y +z
        // 1 = -x +y +z
        // 2 = +x -y +z
        // 3 = -x -y +z
        // 4 = +x +y -z
        // 5 = -x +y -z
        // 6 = +x -y -z
        // 7 = -x -y -z
        [
            Compartment::new(
                0,
                vec![
                    Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0),
                ],
            ),
            Compartment::new(
                1,
                vec![
                    Plane::new(GA3D::new(-1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0),
                ],
            ),
            Compartment::new(
                2,
                vec![
                    Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, -1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0),
                ],
            ),
            Compartment::new(
                3,
                vec![
                    Plane::new(GA3D::new(-1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, -1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, 1.0), 0.0),
                ],
            ),
            Compartment::new(
                4,
                vec![
                    Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, -1.0), 0.0),
                ],
            ),
            Compartment::new(
                5,
                vec![
                    Plane::new(GA3D::new(-1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, -1.0), 0.0),
                ],
            ),
            Compartment::new(
                6,
                vec![
                    Plane::new(GA3D::new(1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, -1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, -1.0), 0.0),
                ],
            ),
            Compartment::new(
                7,
                vec![
                    Plane::new(GA3D::new(-1.0, 0.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, -1.0, 0.0), 0.0),
                    Plane::new(GA3D::new(0.0, 0.0, -1.0), 0.0),
                ],
            ),
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

#[derive(Clone, Debug)]
pub struct Plane {
    normal: GA3D,
    offset: f64,
}

impl Plane {
    pub fn new(normal: GA3D, offset: f64) -> Self {
        Plane {
            normal: normal.normalize(),
            offset,
        }
    }

    pub fn contains(&self, point: &GA3D, tolerance: f64) -> bool {
        let distance = self.normal.dot(point) - self.offset;
        distance.abs() < tolerance
    }
}

#[derive(Debug, Clone)]
pub struct Compartment {
    id: usize,
    boundaries: Vec<Plane>,
}

impl Compartment {
    pub fn new(id: usize, boundaries: Vec<Plane>) -> Self {
        Compartment { id, boundaries }
    }

    pub fn contains(&self, position: &GA3D) -> bool {
        self.boundaries
            .iter()
            .all(|plane| plane.normal.dot(position) - plane.offset > 0.0)
    }
}

pub struct SpaceTimeBase {
    pub spatial_dims: usize,
    pub temporal_dim: usize,
    pub geometry: CubeGeometry,
}
