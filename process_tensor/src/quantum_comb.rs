// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// quantum_comb.rs — Quantum Combs — causal composition of quantum operations
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 17-10-2024
// All rights reserved.
// ---------------------------------------------------------------------------

use crate::quantum_channel::QuantumChannel;
use nalgebra::DMatrix;
use num_complex::Complex64;
use uuid::Uuid;

// ============================================================================
// QUANTUM COMB
// ============================================================================

/// Quantum comb - causal composition of channels
#[derive(Debug, Clone)]
pub struct QuantumComb {
    /// Unique identifier
    pub id: Uuid,
    
    /// Channels in causal order
    pub channels: Vec<QuantumChannel>,
    
    /// Causal ordering
    pub causal_order: CausalOrder,
    
    /// Number of slots
    pub num_slots: usize,
    
    /// System dimension at each slot
    pub slot_dimensions: Vec<usize>,
}

impl QuantumComb {
    /// Create new quantum comb
    pub fn new(num_slots: usize) -> Self {
        Self {
            id: Uuid::new_v4(),
            channels: Vec::new(),
            causal_order: CausalOrder::new(num_slots),
            num_slots,
            slot_dimensions: vec![2; num_slots], // Default: qubits
        }
    }
    
    /// Insert channel at slot
    pub fn insert_channel(&mut self, slot: usize, channel: QuantumChannel) -> Result<(), String> {
        if slot >= self.num_slots {
            return Err(format!("Slot {} out of bounds", slot));
        }
        
        // Verify causal order
        if !self.causal_order.can_insert(slot) {
            return Err("Violates causal order".to_string());
        }
        
        self.channels.push(channel);
        self.causal_order.mark_filled(slot);
        
        Ok(())
    }
    
    /// Apply comb to initial state
    pub fn apply(&self, initial_state: &DMatrix<Complex64>) -> DMatrix<Complex64> {
        let mut state = initial_state.clone();
        
        // Apply channels in causal order
        for slot in self.causal_order.get_order() {
            if let Some(channel) = self.get_channel_at_slot(slot) {
                state = channel.apply(&state);
            }
        }
        
        state
    }
    
    /// Get channel at specific slot
    fn get_channel_at_slot(&self, slot: usize) -> Option<&QuantumChannel> {
        // Simplified - would track slot→channel mapping
        self.channels.get(slot)
    }
    
    /// Compute link product with another comb
    pub fn link_product(&self, other: &QuantumComb) -> Result<QuantumComb, String> {
        // Verify compatibility
        if self.num_slots == 0 || other.num_slots == 0 {
            return Err("Empty combs cannot be linked".to_string());
        }
        
        let mut linked = QuantumComb::new(self.num_slots + other.num_slots - 1);
        
        // Copy channels from first comb
        for (i, channel) in self.channels.iter().enumerate() {
            linked.insert_channel(i, channel.clone())?;
        }
        
        // Append channels from second comb
        for (i, channel) in other.channels.iter().enumerate() {
            linked.insert_channel(self.num_slots + i, channel.clone())?;
        }
        
        Ok(linked)
    }
    
    /// Check if comb is valid (respects causality)
    pub fn is_valid(&self) -> bool {
        // Check that all channels respect causal structure
        self.causal_order.is_consistent()
    }
    
    /// Convert to process tensor
    pub fn to_process_tensor(&self) -> crate::process_tensor::ProcessTensor {
        let mut pt = crate::process_tensor::ProcessTensor::new(
            self.slot_dimensions[0],
            self.num_slots,
        );
        
        for (idx, channel) in self.channels.iter().enumerate() {
            let step = crate::process_tensor::TemporalStep::new(
                idx as f64,
                channel.clone(),
            );
            pt.add_step(step);
        }
        
        pt
    }
}

// ============================================================================
// CAUSAL ORDER
// ============================================================================

/// Causal ordering of operations
#[derive(Debug, Clone)]
pub struct CausalOrder {
    /// Number of time slots
    pub num_slots: usize,
    
    /// Precedence relations: (earlier, later)
    pub precedence: Vec<(usize, usize)>,
    
    /// Slots that are filled
    pub filled_slots: Vec<bool>,
}

impl CausalOrder {
    /// Create new causal order
    pub fn new(num_slots: usize) -> Self {
        Self {
            num_slots,
            precedence: Vec::new(),
            filled_slots: vec![false; num_slots],
        }
    }
    
    /// Add precedence constraint
    pub fn add_precedence(&mut self, earlier: usize, later: usize) -> Result<(), String> {
        if earlier >= self.num_slots || later >= self.num_slots {
            return Err("Slot index out of bounds".to_string());
        }
        
        if earlier >= later {
            return Err("Invalid precedence: earlier must be < later".to_string());
        }
        
        // Check for cycles
        if self.creates_cycle(earlier, later) {
            return Err("Precedence creates cycle".to_string());
        }
        
        self.precedence.push((earlier, later));
        Ok(())
    }
    
    /// Check if slot can be filled
    pub fn can_insert(&self, slot: usize) -> bool {
        // Check if all precedent slots are filled
        for &(earlier, later) in &self.precedence {
            if later == slot && !self.filled_slots[earlier] {
                return false;
            }
        }
        true
    }
    
    /// Mark slot as filled
    pub fn mark_filled(&mut self, slot: usize) {
        if slot < self.num_slots {
            self.filled_slots[slot] = true;
        }
    }
    
    /// Get topological ordering
    pub fn get_order(&self) -> Vec<usize> {
        // Simple topological sort
        let mut order = Vec::new();
        let mut visited = vec![false; self.num_slots];
        
        for slot in 0..self.num_slots {
            if !visited[slot] {
                self.dfs_visit(slot, &mut visited, &mut order);
            }
        }
        
        order.reverse();
        order
    }
    
    /// DFS visit for topological sort
    fn dfs_visit(&self, slot: usize, visited: &mut [bool], order: &mut Vec<usize>) {
        visited[slot] = true;
        
        // Visit all successors
        for &(earlier, later) in &self.precedence {
            if earlier == slot && !visited[later] {
                self.dfs_visit(later, visited, order);
            }
        }
        
        order.push(slot);
    }
    
    /// Check if adding edge creates cycle
    fn creates_cycle(&self, from: usize, to: usize) -> bool {
        // Check if there's already a path from 'to' to 'from'
        let mut visited = vec![false; self.num_slots];
        self.has_path(to, from, &mut visited)
    }
    
    /// Check if path exists
    fn has_path(&self, from: usize, to: usize, visited: &mut [bool]) -> bool {
        if from == to {
            return true;
        }
        
        visited[from] = true;
        
        for &(earlier, later) in &self.precedence {
            if earlier == from && !visited[later] {
                if self.has_path(later, to, visited) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// Check if causal order is consistent
    pub fn is_consistent(&self) -> bool {
        // No cycles
        for i in 0..self.num_slots {
            let mut visited = vec![false; self.num_slots];
            if self.has_path(i, i, &mut visited) {
                return false;
            }
        }
        true
    }
}

// ============================================================================
// LINK PRODUCT
// ============================================================================

/// Link product operation for combs
pub struct LinkProduct;

impl LinkProduct {
    /// Perform link product: C₁ ⋄ C₂
    pub fn apply(
        comb1: &QuantumComb,
        comb2: &QuantumComb,
    ) -> Result<QuantumComb, String> {
        comb1.link_product(comb2)
    }
    
    /// Check if combs are compatible for linking
    pub fn are_compatible(comb1: &QuantumComb, comb2: &QuantumComb) -> bool {
        // Check dimension compatibility
        if let Some(last_dim1) = comb1.slot_dimensions.last() {
            if let Some(first_dim2) = comb2.slot_dimensions.first() {
                return last_dim1 == first_dim2;
            }
        }
        false
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_quantum_comb_creation() {
        let comb = QuantumComb::new(3);
        assert_eq!(comb.num_slots, 3);
        assert!(comb.is_valid());
    }
    
    #[test]
    fn test_causal_order() {
        let mut order = CausalOrder::new(4);
        
        assert!(order.add_precedence(0, 1).is_ok());
        assert!(order.add_precedence(1, 2).is_ok());
        assert!(order.add_precedence(2, 3).is_ok());
        
        // Should detect cycle
        assert!(order.add_precedence(3, 0).is_err());
        
        assert!(order.is_consistent());
    }
    
    #[test]
    fn test_topological_order() {
        let mut order = CausalOrder::new(4);
        order.add_precedence(0, 2).unwrap();
        order.add_precedence(1, 2).unwrap();
        order.add_precedence(2, 3).unwrap();
        
        let topo_order = order.get_order();
        assert_eq!(topo_order.len(), 4);
        
        // 0 and 1 must come before 2
        let pos_0 = topo_order.iter().position(|&x| x == 0).unwrap();
        let pos_1 = topo_order.iter().position(|&x| x == 1).unwrap();
        let pos_2 = topo_order.iter().position(|&x| x == 2).unwrap();
        
        assert!(pos_0 < pos_2);
        assert!(pos_1 < pos_2);
    }
}
