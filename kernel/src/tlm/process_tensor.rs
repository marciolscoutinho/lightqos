//! Process Tensors - Non-Markovian Quantum Memory Modeling
//! 
//! Based on Process Tensor theory and integration with OQuPy

use super::QuantumOperation;
use std::collections::HashMap;
use std::time::Duration;

/// Quantum process memory manager
pub struct ProcessMemory {
    /// History of recent operations
    operation_history: Vec<OperationRecord>,
    
    /// Computed process tensors
    process_tensors: HashMap<String, ProcessTensor>,
    
    /// Memory window (how long history is kept)
    memory_window: Duration,
}

impl ProcessMemory {
    pub fn new() -> Self {
        ProcessMemory {
            operation_history: Vec::new(),
            process_tensors: HashMap::new(),
            memory_window: Duration::from_millis(100), // 100 ms of memory
        }
    }
    
    /// Checks whether an operation is compatible with the time slot
    /// Takes memory effects into account (non-Markovian)
    pub fn is_compatible(
        &self,
        operation: &QuantumOperation,
        time_slot: &super::TimeSlot,
    ) -> bool {
        // Gets recent operations that may carry memory
        let recent_ops = self.get_recent_operations(time_slot.start);
        
        // Checks temporal cross-talk
        for past_op in recent_ops {
            if self.has_memory_overlap(operation, &past_op) {
                // Computes the process tensor to verify compatibility
                if let Some(tensor) = self.compute_process_tensor(operation, &past_op) {
                    if !tensor.is_compatible() {
                        return false;
                    }
                }
            }
        }
        
        true
    }
    
    /// Adds an operation to the history
    pub fn record_operation(&mut self, operation: QuantumOperation, execution_time: Duration) {
        self.operation_history.push(OperationRecord {
            operation,
            execution_time,
            timestamp: std::time::Instant::now(),
        });
        
        // Clears old history (outside the memory window)
        self.cleanup_old_history();
    }
    
    /// Gets recent operations (within the memory window)
    fn get_recent_operations(&self, _reference_time: Duration) -> Vec<&OperationRecord> {
        self.operation_history
            .iter()
            .filter(|record| record.timestamp.elapsed() < self.memory_window)
            .collect()
    }
    
    /// Checks whether two operations share qubits (potential memory)
    fn has_memory_overlap(&self, op1: &QuantumOperation, record: &OperationRecord) -> bool {
        op1.qubits.iter().any(|q1| {
            record.operation.qubits.iter().any(|q2| q1 == q2)
        })
    }
    
    /// Computes a process tensor between two operations
    fn compute_process_tensor(
        &mut self,
        current_op: &QuantumOperation,
        past_op: &OperationRecord,
    ) -> Option<ProcessTensor> {
        let key = format!(
            "{}_{}_{}",
            past_op.operation.gate_type,
            current_op.gate_type,
            past_op.timestamp.elapsed().as_nanos()
        );
        
        // Checks cache
        if let Some(tensor) = self.process_tensors.get(&key) {
            return Some(tensor.clone());
        }
        
        // Computes a new tensor (simplified)
        let tensor = ProcessTensor::compute(current_op, &past_op.operation);
        self.process_tensors.insert(key, tensor.clone());
        
        Some(tensor)
    }
    
    /// Cleans old history
    fn cleanup_old_history(&mut self) {
        self.operation_history
            .retain(|record| record.timestamp.elapsed() < self.memory_window);
        
        // Clears cache of old tensors
        self.process_tensors.clear();
    }
}

/// Record of an executed operation
struct OperationRecord {
    operation: QuantumOperation,
    execution_time: Duration,
    timestamp: std::time::Instant,
}

/// Process tensor (simplified representation)
#[derive(Clone)]
pub struct ProcessTensor {
    /// Fidelity of the joint process
    fidelity: f64,
    
    /// Temporal correlation (0 = no memory, 1 = strongly correlated)
    temporal_correlation: f64,
    
    /// Quantum channels involved
    channels: Vec<String>,
}

impl ProcessTensor {
    /// Computes a process tensor between two operations
    fn compute(op1: &QuantumOperation, op2: &QuantumOperation) -> Self {
        // Simplified model: correlation based on qubit overlap
        let shared_qubits: Vec<_> = op1.qubits
            .iter()
            .filter(|q| op2.qubits.contains(q))
            .collect();
        
        let temporal_correlation =
            shared_qubits.len() as f64 / op1.qubits.len().max(1) as f64;
        
        // Fidelity decays with temporal correlation
        let fidelity = 0.99 * (1.0 - temporal_correlation * 0.1);
        
        ProcessTensor {
            fidelity,
            temporal_correlation,
            channels: shared_qubits.iter().map(|s| s.to_string()).collect(),
        }
    }
    
    /// Checks whether the tensor allows compatible execution
    fn is_compatible(&self) -> bool {
        // Compatibility threshold
        self.fidelity > 0.95 && self.temporal_correlation < 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_memory() {
        let mut memory = ProcessMemory::new();
        
        let op1 = QuantumOperation {
            gate_type: "H".to_string(),
            qubits: vec!["q0".to_string()],
            params: vec![],
        };
        
        memory.record_operation(op1, Duration::from_nanos(50));
        
        assert_eq!(memory.operation_history.len(), 1);
    }
}
