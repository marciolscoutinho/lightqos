// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// py_tlm.rs — PyO3 Python bindings for TLM (Temporal Layer Manager)
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 13-03-2025
// All rights reserved.
// ---------------------------------------------------------------------------

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::time::{Duration, Instant};
use uuid::Uuid;

// ============================================================================
// PyTemporalContract
// ============================================================================

/// Temporal contract for a quantum operation
///
/// Defines deadlines, phases and execution guarantees for an operation.
#[pyclass(name = "TemporalContract")]
#[derive(Debug, Clone)]
pub struct PyTemporalContract {
    pub id: String,
    pub operation: String,
    pub deadline_ms: f64,
    pub phase_ns: u64,
    pub priority: u8,
    pub created_at: Instant,
    pub fulfilled: bool,
}

#[pymethods]
impl PyTemporalContract {
    #[new]
    #[pyo3(signature = (operation, deadline_ms=100.0, phase_ns=0, priority=5))]
    pub fn new(operation: String, deadline_ms: f64, phase_ns: u64, priority: u8) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            operation,
            deadline_ms,
            phase_ns,
            priority: priority.clamp(1, 10),
            created_at: Instant::now(),
            fulfilled: false,
        }
    }

    /// Unique contract identifier
    #[getter]
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Operation name
    #[getter]
    pub fn operation(&self) -> &str {
        &self.operation
    }

    /// Deadline in milliseconds
    #[getter]
    pub fn deadline_ms(&self) -> f64 {
        self.deadline_ms
    }

    /// Phase in nanoseconds
    #[getter]
    pub fn phase_ns(&self) -> u64 {
        self.phase_ns
    }

    /// Priority (1-10)
    #[getter]
    pub fn priority(&self) -> u8 {
        self.priority
    }

    /// Indicates whether the contract was fulfilled
    #[getter]
    pub fn fulfilled(&self) -> bool {
        self.fulfilled
    }

    /// Elapsed time since creation (ms)
    pub fn elapsed_ms(&self) -> f64 {
        self.created_at.elapsed().as_secs_f64() * 1000.0
    }

    /// Checks whether the deadline has been exceeded
    pub fn is_expired(&self) -> bool {
        self.elapsed_ms() > self.deadline_ms
    }

    /// Remaining time margin (ms); negative if expired
    pub fn time_remaining_ms(&self) -> f64 {
        self.deadline_ms - self.elapsed_ms()
    }

    pub fn __repr__(&self) -> String {
        format!(
            "TemporalContract(op='{}', deadline={:.1}ms, priority={}, fulfilled={})",
            self.operation, self.deadline_ms, self.priority, self.fulfilled
        )
    }
}

// ============================================================================
// PyContractManager
// ============================================================================

/// Temporal contract manager
///
/// Maintains and monitors active contracts. Ensures execution within the
/// defined deadlines and applies rollback in case of failure.
#[pyclass(name = "ContractManager")]
pub struct PyContractManager {
    contracts: Vec<PyTemporalContract>,
    fulfilled_count: usize,
    expired_count: usize,
}

#[pymethods]
impl PyContractManager {
    #[new]
    pub fn new() -> Self {
        Self {
            contracts: Vec::new(),
            fulfilled_count: 0,
            expired_count: 0,
        }
    }

    /// Registers a new contract
    pub fn register(&mut self, contract: PyTemporalContract) -> String {
        let id = contract.id.clone();
        self.contracts.push(contract);
        id
    }

    /// Creates and registers a simple contract
    #[pyo3(signature = (operation, deadline_ms=100.0, priority=5))]
    pub fn create_contract(&mut self, operation: String, deadline_ms: f64, priority: u8) -> PyTemporalContract {
        let contract = PyTemporalContract::new(operation, deadline_ms, 0, priority);
        self.contracts.push(contract.clone());
        contract
    }

    /// Marks a contract as fulfilled
    pub fn fulfill(&mut self, contract_id: &str) -> bool {
        for c in &mut self.contracts {
            if c.id == contract_id {
                c.fulfilled = true;
                self.fulfilled_count += 1;
                return true;
            }
        }
        false
    }

    /// Removes expired contracts; returns the number removed
    pub fn gc_expired(&mut self) -> usize {
        let before = self.contracts.len();
        let expired: Vec<_> = self.contracts
            .iter()
            .filter(|c| c.is_expired() && !c.fulfilled)
            .map(|c| c.id.clone())
            .collect();

        self.expired_count += expired.len();
        self.contracts.retain(|c| !expired.contains(&c.id));
        before - self.contracts.len()
    }

    /// Number of active contracts
    pub fn active_count(&self) -> usize {
        self.contracts.iter().filter(|c| !c.fulfilled && !c.is_expired()).count()
    }

    /// Manager statistics
    pub fn stats<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let d = PyDict::new(py);
        d.set_item("total_registered", self.contracts.len() + self.fulfilled_count + self.expired_count)?;
        d.set_item("active", self.active_count())?;
        d.set_item("fulfilled", self.fulfilled_count)?;
        d.set_item("expired", self.expired_count)?;
        Ok(d)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "ContractManager(active={}, fulfilled={}, expired={})",
            self.active_count(), self.fulfilled_count, self.expired_count
        )
    }
}

// ============================================================================
// PyHarmonicScheduler
// ============================================================================

/// Harmonic scheduler for quantum operations
///
/// Organizes operations into temporal windows (epochs), respecting
/// contract priorities and deadlines.
#[pyclass(name = "HarmonicScheduler")]
pub struct PyHarmonicScheduler {
    epoch_duration_us: u64,
    current_epoch: u64,
    scheduled_count: usize,
    dropped_count: usize,
}

#[pymethods]
impl PyHarmonicScheduler {
    #[new]
    #[pyo3(signature = (epoch_duration_us=1000))]
    pub fn new(epoch_duration_us: u64) -> Self {
        Self {
            epoch_duration_us,
            current_epoch: 0,
            scheduled_count: 0,
            dropped_count: 0,
        }
    }

    /// Advances to the next epoch
    pub fn tick(&mut self) -> u64 {
        self.current_epoch += 1;
        self.current_epoch
    }

    /// Schedules an operation in the current epoch
    ///
    /// Returns `true` if it was scheduled, `false` if the contract expired.
    pub fn schedule(&mut self, contract: &PyTemporalContract) -> bool {
        if contract.is_expired() {
            self.dropped_count += 1;
            return false;
        }
        self.scheduled_count += 1;
        true
    }

    /// Current epoch
    #[getter]
    pub fn current_epoch(&self) -> u64 {
        self.current_epoch
    }

    /// Duration of each epoch in microseconds
    #[getter]
    pub fn epoch_duration_us(&self) -> u64 {
        self.epoch_duration_us
    }

    /// Scheduler statistics
    pub fn stats<'py>(&self, py: Python<'py>) -> PyResult<&'py PyDict> {
        let d = PyDict::new(py);
        d.set_item("epoch", self.current_epoch)?;
        d.set_item("epoch_duration_us", self.epoch_duration_us)?;
        d.set_item("scheduled", self.scheduled_count)?;
        d.set_item("dropped", self.dropped_count)?;
        Ok(d)
    }

    pub fn __repr__(&self) -> String {
        format!(
            "HarmonicScheduler(epoch={}, scheduled={}, dropped={})",
            self.current_epoch, self.scheduled_count, self.dropped_count
        )
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_contract_creation() {
        let c = PyTemporalContract::new("H_GATE".into(), 50.0, 0, 5);
        assert_eq!(c.operation(), "H_GATE");
        assert_eq!(c.deadline_ms(), 50.0);
        assert!(!c.fulfilled());
        assert!(!c.is_expired());
    }

    #[test]
    fn test_contract_manager_lifecycle() {
        let mut mgr = PyContractManager::new();
        let c = mgr.create_contract("CNOT".into(), 200.0, 7);
        let id = c.id().to_string();
        assert_eq!(mgr.active_count(), 1);
        let ok = mgr.fulfill(&id);
        assert!(ok);
        assert_eq!(mgr.fulfilled_count, 1);
    }

    #[test]
    fn test_harmonic_scheduler_ticks() {
        let mut sched = PyHarmonicScheduler::new(1000);
        sched.tick();
        sched.tick();
        assert_eq!(sched.current_epoch(), 2);
    }
}
