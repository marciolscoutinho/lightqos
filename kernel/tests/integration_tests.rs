#[cfg(test)]
mod integration_tests {
    use lightqos_kernel::efal::EtherField;
    use lightqos_kernel::emf::EntanglementFabric;
    use lightqos_kernel::tlm::TemporalLayerManager;
    
    #[test]
    fn test_efal_channel_creation() {
        let config = lightqos_kernel::efal::HardwareConfig {
            platform: "simulator".to_string(),
            num_qubits: 5,
            connectivity: vec![(0, 1), (1, 2), (2, 3), (3, 4)],
            coherence_times: vec![1e-3; 5],
        };
        
        let mut ether_field = EtherField::new(&config);
        
        // Allocates two defects
        let defect_a = ether_field.allocate_defect(
            lightqos_kernel::math::geometric_algebra::GA3D::new(0.0, 0.0, 0.5),
            lightqos_kernel::efal::defect::DefectType::Qubit,
        ).unwrap();
        
        let defect_b = ether_field.allocate_defect(
            lightqos_kernel::math::geometric_algebra::GA3D::new(1.0, 0.0, 0.5),
            lightqos_kernel::efal::defect::DefectType::Qubit,
        ).unwrap();
        
        // Creates a channel between them
        let channel_id = ether_field.create_channel(
            &defect_a,
            &defect_b,
            lightqos_kernel::efal::channel::ChannelType::Optical,
        ).unwrap();
        
        assert!(ether_field.channels.contains_key(&channel_id));
    }
    
    #[test]
    fn test_emf_bell_pair_allocation() {
        let config = lightqos_kernel::efal::HardwareConfig {
            platform: "simulator".to_string(),
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
    fn test_tlm_harmonic_scheduling() {
        let mut tlm = TemporalLayerManager::new();
        
        let operation = lightqos_kernel::tlm::QuantumOperation {
            gate_type: "H".to_string(),
            qubits: vec!["dt_0".to_string()],
            params: vec![],
        };
        
        let contract = lightqos_kernel::tlm::contract::TemporalContract {
            max_latency: std::time::Duration::from_nanos(100),
            deadline_phase: 0.1,
            rollback_on_violation: true,
        };
        
        let op_id = tlm.schedule_operation(operation, contract).unwrap();
        
        assert!(!op_id.is_empty());
    }
}
