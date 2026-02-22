//! Full EFAL tests

use lightqos_kernel::efal::*;
use lightqos_kernel::math::geometric_algebra::GA3D;

#[test]
fn test_ether_field_initialization() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 3,
        connectivity: vec![(0, 1), (1, 2)],
        coherence_times: vec![1e-3; 3],
    };
    
    let field = EtherField::new(&config);
    
    assert_eq!(field.channels.len(), 0);
    assert_eq!(field.defects.len(), 0);
}

#[test]
fn test_defect_allocation() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 5,
        connectivity: vec![],
        coherence_times: vec![1e-3; 5],
    };
    
    let mut field = EtherField::new(&config);
    
    // Allocates a defect at a valid position
    let defect_id = field.allocate_defect(
        GA3D::new(1.0, 1.0, 1.0),
        defect::DefectType::Qubit,
    ).unwrap();
    
    assert!(field.defects.contains_key(&defect_id));
    
    // Tries to allocate on an inertial plane (should fail)
    let result = field.allocate_defect(
        GA3D::new(0.0, 0.0, 0.0), // On the south plane
        defect::DefectType::Qubit,
    );
    
    assert!(result.is_err());
}

#[test]
fn test_channel_creation() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 5,
        connectivity: vec![(0, 1), (1, 2)],
        coherence_times: vec![1e-3; 5],
    };
    
    let mut field = EtherField::new(&config);
    
    // Creates two defects
    let dt_a = field.allocate_defect(
        GA3D::new(0.5, 0.5, 0.5),
        defect::DefectType::Qubit,
    ).unwrap();
    
    let dt_b = field.allocate_defect(
        GA3D::new(1.5, 0.5, 0.5),
        defect::DefectType::Qubit,
    ).unwrap();
    
    // Creates a channel between them
    let channel_id = field.create_channel(
        &dt_a,
        &dt_b,
        channel::ChannelType::Optical,
    ).unwrap();
    
    assert!(field.channels.contains_key(&channel_id));
    
    let channel = field.channels.get(&channel_id).unwrap();
    assert_eq!(channel.source, dt_a);
    assert_eq!(channel.target, dt_b);
}

#[test]
fn test_geometry_cube_compartments() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 8,
        connectivity: vec![],
        coherence_times: vec![1e-3; 8],
    };
    
    let field = EtherField::new(&config);
    
    // Tests points in different compartments
    let point1 = GA3D::new(0.5, 0.5, 0.5);   // Compartment 0
    let point2 = GA3D::new(-0.5, 0.5, 0.5);  // Compartment 1
    
    let comp1 = field.geometry.get_compartment(&point1);
    let comp2 = field.geometry.get_compartment(&point2);
    
    assert!(comp1.is_some());
    assert!(comp2.is_some());
    assert_ne!(comp1, comp2);
}

#[test]
fn test_channel_metrics() {
    let path = vec![
        GA3D::new(0.0, 0.0, 0.0),
        GA3D::new(1.0, 0.0, 0.0),
        GA3D::new(1.0, 1.0, 0.0),
    ];
    
    let mut channel = channel::Channel::new(
        "dt_0".to_string(),
        "dt_1".to_string(),
        channel::ChannelType::Optical,
        path,
    );
    
    // Path: 1.0 + 1.0 = 2.0
    assert!((channel.path_length() - 2.0).abs() < 1e-6);
    
    // Tests metrics update
    let new_metrics = channel::ChannelMetrics {
        fidelity: 0.85,
        latency: std::time::Duration::from_nanos(200),
        attenuation_db: 0.5,
        coherence_time: std::time::Duration::from_micros(50),
    };
    
    channel.update_metrics(new_metrics);
    
    assert_eq!(channel.status, channel::ChannelStatus::Degraded);
}
