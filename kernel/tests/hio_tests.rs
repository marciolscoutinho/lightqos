//! Holographic I/O tests

use lightqos_kernel::hio::*;

#[test]
fn test_shadow_collection() {
    let qubits = vec!["q0".to_string(), "q1".to_string()];
    let shadows = shadow_copy::collect_shadows(&qubits, 1000).unwrap();
    
    assert_eq!(shadows.total_samples, 1000);
    assert!(shadows.entropy >= 0.0);
    assert!(shadows.entropy <= 2.0); // Max for 2 qubits
}

#[test]
fn test_multi_base_measurement() {
    let qubits = vec!["q0".to_string()];
    let bases = vec![
        MeasurementBasis::Z,
        MeasurementBasis::X,
        MeasurementBasis::Y,
    ];
    
    let views = observable_view::measure_multi_base(&qubits, &bases).unwrap();
    
    assert_eq!(views.views.len(), 3);
    assert!(views.views.contains_key("Z"));
    assert!(views.views.contains_key("X"));
    assert!(views.views.contains_key("Y"));
}

#[test]
fn test_statistical_guarantee() {
    let qubits = vec!["q0".to_string()];
    
    let shadows = shadow_copy::collect_shadows(&qubits, 2000).unwrap();
    let views = observable_view::measure_multi_base(
        &qubits,
        &[MeasurementBasis::Z],
    ).unwrap();
    
    let config = PrecisionConfig {
        min_samples: 1000,
        max_error: 0.05,
        confidence_level: 0.95,
    };
    
    let guarantee = statistical_guarantee::compute_guarantee(
        &shadows,
        &views,
        &config,
    ).unwrap();
    
    assert_eq!(guarantee.confidence_level, 0.95);
    assert!(guarantee.meets_requirements);
}

#[test]
fn test_holographic_measurement() {
    let mut hio = HolographicIO::new();
    
    let qubits = vec!["q0".to_string(), "q1".to_string()];
    let config = MeasurementConfig::default();
    
    let measurement = hio.measure_holographic(&qubits, config).unwrap();
    
    assert_eq!(measurement.qubits.len(), 2);
    assert!(measurement.guarantee.meets_requirements);
}

#[test]
fn test_required_samples_calculation() {
    let n = statistical_guarantee::required_samples(0.01, 0.95);
    
    // For 95% confidence and 1% error: n ≈ 9604
    assert!(n > 9000);
    assert!(n < 10000);
}
