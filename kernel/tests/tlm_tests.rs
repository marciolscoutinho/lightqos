//! Temporal Layer Manager tests

use lightqos_kernel::tlm::*;
use std::time::Duration;

#[test]
fn test_temporal_contract_validation() {
    let valid = contract::TemporalContract {
        max_latency: Duration::from_nanos(100),
        deadline_phase: 0.1,
        rollback_on_violation: true,
        max_retries: 3,
    };
    
    assert!(valid.validate().is_ok());
    
    let invalid = contract::TemporalContract {
        max_latency: Duration::ZERO,
        deadline_phase: 0.1,
        rollback_on_violation: false,
        max_retries: 1,
    };
    
    assert!(invalid.validate().is_err());
}

#[test]
fn test_contract_violation() {
    let contract = contract::TemporalContract::strict();
    
    assert!(!contract.is_violated_by(Duration::from_nanos(50)));
    assert!(contract.is_violated_by(Duration::from_nanos(200)));
}

#[test]
fn test_harmonic_scheduling() {
    use lightqos_kernel::math::octave_algebra::OctavePosition;
    
    let scheduler = harmonic_scheduler::HarmonicScheduler::new();
    
    // Tests harmonic multipliers
    let pos = OctavePosition::Generation4Plus;
    assert_eq!(pos.harmonic_multiplier(), 16.0);
    
    let pos = OctavePosition::Inertia0;
    assert_eq!(pos.harmonic_multiplier(), 1.0);
}

#[test]
fn test_snapshot_system() {
    use std::collections::VecDeque;
    
    let mut snapshot_system = snapshot::SnapshotSystem::new();
    let mut queue = VecDeque::new();
    
    // Adds an operation
    let op = ScheduledOperation {
        operation: QuantumOperation {
            gate_type: "H".to_string(),
            qubits: vec!["q0".to_string()],
            params: vec![],
        },
        contract: contract::TemporalContract::default_permissive(),
        time_slot: TimeSlot {
            start: Duration::from_nanos(0),
            end: Duration::from_nanos(100),
        },
        snapshot_id: "".to_string(),
    };
    
    queue.push_back(op);
    
    // Creates a snapshot
    let snap_id = snapshot_system.create_snapshot(&queue);
    
    // Modifies the queue
    queue.clear();
    assert_eq!(queue.len(), 0);
    
    // Restores
    snapshot_system.restore(&snap_id, &mut queue);
    assert_eq!(queue.len(), 1);
}

#[test]
fn test_process_memory() {
    let mut memory = process_tensor::ProcessMemory::new();
    
    let op = QuantumOperation {
        gate_type: "CNOT".to_string(),
        qubits: vec!["q0".to_string(), "q1".to_string()],
        params: vec![],
    };
    
    memory.record_operation(op, Duration::from_nanos(50));
    
    assert_eq!(memory.operation_history.len(), 1);
}

#[test]
fn test_tlm_scheduling() {
    let mut tlm = TemporalLayerManager::new();
    
    let operation = QuantumOperation {
        gate_type: "H".to_string(),
        qubits: vec!["dt_0".to_string()],
        params: vec![],
    };
    
    let contract = contract::TemporalContract {
        max_latency: Duration::from_nanos(100),
        deadline_phase: 0.1,
        rollback_on_violation: true,
        max_retries: 3,
    };
    
    let op_id = tlm.schedule_operation(operation, contract).unwrap();
    
    assert!(!op_id.is_empty());
    assert_eq!(tlm.operation_queue.len(), 1);
}
