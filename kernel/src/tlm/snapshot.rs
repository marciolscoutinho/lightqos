//! Snapshot System for Temporal Rollback

use super::ScheduledOperation;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

/// Snapshot system for recovery
pub struct SnapshotSystem {
    /// Stored snapshots
    snapshots: HashMap<String, Snapshot>,
    
    /// Maximum number of snapshots to keep
    max_snapshots: usize,
    
    /// Creation order (for LRU)
    creation_order: VecDeque<String>,
}

impl SnapshotSystem {
    pub fn new() -> Self {
        SnapshotSystem {
            snapshots: HashMap::new(),
            max_snapshots: 100,
            creation_order: VecDeque::new(),
        }
    }
    
    /// Creates a snapshot of the current queue state
    pub fn create_snapshot(&mut self, queue: &VecDeque<ScheduledOperation>) -> String {
        let snapshot_id = format!("snap_{}", Instant::now().elapsed().as_nanos());
        
        let snapshot = Snapshot {
            id: snapshot_id.clone(),
            timestamp: Instant::now(),
            queue_state: queue.iter().map(|op| op.clone()).collect(),
            metadata: SnapshotMetadata::default(),
        };
        
        self.snapshots.insert(snapshot_id.clone(), snapshot);
        self.creation_order.push_back(snapshot_id.clone());
        
        // Removes old snapshots if the limit is exceeded
        if self.snapshots.len() > self.max_snapshots {
            if let Some(oldest_id) = self.creation_order.pop_front() {
                self.snapshots.remove(&oldest_id);
            }
        }
        
        snapshot_id
    }
    
    /// Restores state from a snapshot
    pub fn restore(&self, snapshot_id: &str, queue: &mut VecDeque<ScheduledOperation>) {
        if let Some(snapshot) = self.snapshots.get(snapshot_id) {
            queue.clear();
            for op in &snapshot.queue_state {
                queue.push_back(op.clone());
            }
        }
    }
    
    /// Deletes a snapshot
    pub fn delete_snapshot(&mut self, snapshot_id: &str) {
        self.snapshots.remove(snapshot_id);
        self.creation_order.retain(|id| id != snapshot_id);
    }
    
    /// Lists all available snapshots
    pub fn list_snapshots(&self) -> Vec<SnapshotInfo> {
        self.snapshots
            .values()
            .map(|snap| SnapshotInfo {
                id: snap.id.clone(),
                timestamp: snap.timestamp,
                queue_size: snap.queue_state.len(),
            })
            .collect()
    }
}

/// Snapshot of the system state
#[derive(Clone)]
struct Snapshot {
    id: String,
    timestamp: Instant,
    queue_state: Vec<ScheduledOperation>,
    metadata: SnapshotMetadata,
}

#[derive(Clone, Default)]
struct SnapshotMetadata {
    reason: String,
    user_tag: Option<String>,
}

/// Snapshot info (for listing)
pub struct SnapshotInfo {
    pub id: String,
    pub timestamp: Instant,
    pub queue_size: usize,
}
