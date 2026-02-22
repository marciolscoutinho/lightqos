//! Degraded Entanglement Recycler

use std::collections::HashSet;

pub struct EntanglementRecycler {
    marked_pairs: HashSet<String>,
}

impl EntanglementRecycler {
    pub fn new() -> Self {
        EntanglementRecycler {
            marked_pairs: HashSet::new(),
        }
    }
    
    pub fn mark_for_recycling(&mut self, pair_id: &str) {
        self.marked_pairs.insert(pair_id.to_string());
    }
    
    pub fn get_marked_pairs(&self) -> Vec<String> {
        self.marked_pairs.iter().cloned().collect()
    }
    
    pub fn clear_marked(&mut self) {
        self.marked_pairs.clear();
    }
}
