//! Entangled Memory Fabric tests

use lightqos_kernel::emf::*;
use lightqos_kernel::efal::*;
use std::time::Duration;

#[test]
fn test_bell_pair_creation() {
    let pair = entanglement_pool::BellPair::create_via_route(
        "dt_0",
        "dt_1",
        pser_routing::PSERRoute {
            waypoints: vec!["dt_0".to_string(), "dt_1".to_string()],
            total_cost: 1.0,
            pressure_profile: vec![1.0, 1.0],
        },
        0.99,
    ).unwrap();
    
    assert_eq!(pair.qubit_a, "dt_0");
    assert_eq!(pair.qubit_b, "dt_1");
    assert!(pair.fidelity >= 0.98);
}

#[test]
fn test_bell_pair_coherence() {
    let mut pair = entanglement_pool::BellPair::create_via_route(
        "dt_0",
        "dt_1",
        pser_routing::PSERRoute {
            waypoints: vec!["dt_0".to_string(), "dt_1".to_string()],
            total_cost: 1.0,
            pressure_profile: vec![],
        },
        0.99,
    ).unwrap();
    
    assert!(pair.is_coherent());
    
    // Forces expiration
    pair.lifetime = Duration::from_nanos(1);
    std::thread::sleep(Duration::from_millis(10));
    
    assert!(!pair.is_coherent());
}

#[test]
fn test_ergotropy_calculation() {
    let pair = entanglement_pool::BellPair::create_via_route(
        "dt_0",
        "dt_1",
        pser_routing::PSERRoute {
            waypoints: vec![],
            total_cost: 1.0,
            pressure_profile: vec![],
        },
        0.95,
    ).unwrap();
    
    let ergotropy = pair.ergotropy();
    
    // W_max ≈ 2F - 1 = 2(0.95) - 1 = 0.9
    assert!((ergotropy - 0.9).abs() < 0.05);
}

#[test]
fn test_emf_allocation() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 2,
        connectivity: vec![(0, 1)],
        coherence_times: vec![1e-3; 2],
    };
    
    let ether_field = EtherField::new(&config);
    let mut emf = EntanglementFabric::new(&ether_field);
    
    let pair_id = emf.allocate_bell_pair("dt_0", "dt_1", 0.99).unwrap();
    
    assert!(emf.bell_pairs.contains_key(&pair_id));
}

#[test]
fn test_emf_recycling() {
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 2,
        connectivity: vec![(0, 1)],
        coherence_times: vec![1e-3; 2],
    };
    
    let ether_field = EtherField::new(&config);
    let mut emf = EntanglementFabric::new(&ether_field);
    
    // Allocates a low-quality pair
    let pair_id = emf.allocate_bell_pair("dt_0", "dt_1", 0.5).unwrap();
    
    // Marks it for recycling
    emf.recycler.mark_for_recycling(&pair_id);
    
    // Runs recycling
    emf.recycle_degraded_pairs();
    
    assert!(!emf.bell_pairs.contains_key(&pair_id));
}

#[test]
fn test_pser_routing() {
    use lightqos_kernel::efal::geometry::CubeGeometry;
    use lightqos_kernel::efal::topology::DynamicTopology;
    
    let config = HardwareConfig {
        platform: "test".to_string(),
        num_qubits: 4,
        connectivity: vec![(0, 1), (1, 2), (2, 3)],
        coherence_times: vec![1e-3; 4],
    };
    
    let geometry = CubeGeometry::from_hardware(&config);
    let topology = DynamicTopology::new();
    
    // TODO: Complete once DynamicTopology is implemented
}
