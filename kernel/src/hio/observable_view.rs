// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// observable_view.rs — HIO Observable View — expectation value estimation from shadows
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 05-07-2022
// All rights reserved.
// ---------------------------------------------------------------------------

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use nalgebra::{Complex, DMatrix};
use uuid::Uuid;

use crate::hio::shadow_copy::{QuantumShadow, PauliString, PauliOperator};

// ============================================================================
// QUANTUM OBSERVABLE
// ============================================================================

/// Quantum observable (Hermitian operator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observable {
    /// Unique ID
    pub id: Uuid,
    
    /// Observable name
    pub name: String,
    
    /// Representation as a Pauli string
    pub pauli_representation: PauliString,
    
    /// Observable matrix (if available)
    pub matrix: Option<Vec<Complex<f64>>>,
    
    /// Observable type
    pub observable_type: ObservableType,
    
    /// Metadata
    pub metadata: ObservableMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservableType {
    /// Local observable (single-qubit)
    Local,
    
    /// Global observable (multi-qubit)
    Global,
    
    /// Hamiltonian
    Hamiltonian,
    
    /// Stabilizer
    Stabilizer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservableMetadata {
    /// Affected qubits
    pub qubits: Vec<usize>,
    
    /// Weight (number of non-identity Paulis)
    pub weight: usize,
    
    /// Is it diagonal in the computational basis?
    pub diagonal: bool,
    
    /// Is it Hermitian?
    pub hermitian: bool,
}

impl Observable {
    /// Creates an observable from a Pauli string
    pub fn from_pauli(name: String, pauli: PauliString) -> Self {
        let weight = pauli.paulis.iter()
            .filter(|&&p| p != PauliOperator::I)
            .count();
        
        let qubits = pauli.paulis.iter()
            .enumerate()
            .filter_map(|(i, &p)| {
                if p != PauliOperator::I {
                    Some(i)
                } else {
                    None
                }
            })
            .collect();
        
        let diagonal = pauli.paulis.iter()
            .all(|&p| p == PauliOperator::I || p == PauliOperator::Z);
        
        let observable_type = if weight == 1 {
            ObservableType::Local
        } else {
            ObservableType::Global
        };
        
        Self {
            id: Uuid::new_v4(),
            name,
            pauli_representation: pauli.clone(),
            matrix: Some(pauli.to_matrix().iter().copied().collect()),
            observable_type,
            metadata: ObservableMetadata {
                qubits,
                weight,
                diagonal,
                hermitian: true,  // Paulis are always Hermitian
            },
        }
    }
    
    /// Checks whether two observables commute
    pub fn commutes_with(&self, other: &Observable) -> bool {
        // Simplification: check commutativity through Paulis
        let p1 = &self.pauli_representation.paulis;
        let p2 = &other.pauli_representation.paulis;
        
        if p1.len() != p2.len() {
            return false;
        }
        
        // Count anticommutations
        let mut anticommutations = 0;
        
        for (op1, op2) in p1.iter().zip(p2.iter()) {
            if Self::anticommute(*op1, *op2) {
                anticommutations += 1;
            }
        }
        
        // They commute if the number of anticommutations is even
        anticommutations % 2 == 0
    }
    
    fn anticommute(op1: PauliOperator, op2: PauliOperator) -> bool {
        use PauliOperator::*;
        
        match (op1, op2) {
            (I, _) | (_, I) => false,
            (X, Y) | (Y, X) => true,
            (X, Z) | (Z, X) => true,
            (Y, Z) | (Z, Y) => true,
            _ => false,
        }
    }
    
    /// Calculates the expected value using a shadow
    pub fn expectation_value(&self, shadow: &QuantumShadow) -> Result<f64, String> {
        shadow.estimate_observable(&self.pauli_representation)
    }
}

// ============================================================================
// OBSERVABLE VIEW
// ============================================================================

/// View of a set of compatible observables
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservableView {
    /// Unique ID
    pub id: Uuid,
    
    /// View name
    pub name: String,
    
    /// Observables in this view
    pub observables: Vec<Observable>,
    
    /// Measured expected values
    pub measured_values: HashMap<Uuid, f64>,
    
    /// Associated shadow
    pub shadow_id: Option<Uuid>,
    
    /// Configuration
    pub config: ViewConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewConfig {
    /// Enforce commutativity?
    pub enforce_commutativity: bool,
    
    /// Minimum number of measurements
    pub min_measurements: usize,
    
    /// Target precision
    pub target_precision: f64,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            enforce_commutativity: true,
            min_measurements: 100,
            target_precision: 0.01,
        }
    }
}

impl ObservableView {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            observables: Vec::new(),
            measured_values: HashMap::new(),
            shadow_id: None,
            config: ViewConfig::default(),
        }
    }
    
    pub fn with_config(mut self, config: ViewConfig) -> Self {
        self.config = config;
        self
    }
    
    /// Adds an observable to the view
    pub fn add_observable(&mut self, observable: Observable) -> Result<(), String> {
        // Check commutativity with existing observables
        if self.config.enforce_commutativity {
            for existing in &self.observables {
                if !observable.commutes_with(existing) {
                    return Err(format!(
                        "Observable '{}' does not commute with '{}'",
                        observable.name,
                        existing.name
                    ));
                }
            }
        }
        
        self.observables.push(observable);
        Ok(())
    }
    
    /// Measures all observables using a shadow
    pub fn measure_all(&mut self, shadow: &QuantumShadow) -> Result<(), String> {
        self.shadow_id = Some(shadow.id);
        
        for observable in &self.observables {
            let value = observable.expectation_value(shadow)?;
            self.measured_values.insert(observable.id, value);
        }
        
        Ok(())
    }
    
    /// Gets the measured value of an observable
    pub fn get_value(&self, observable_id: Uuid) -> Option<f64> {
        self.measured_values.get(&observable_id).copied()
    }
    
    /// Returns all measured values
    pub fn all_values(&self) -> Vec<(String, f64)> {
        self.observables.iter()
            .filter_map(|obs| {
                self.measured_values.get(&obs.id)
                    .map(|&value| (obs.name.clone(), value))
            })
            .collect()
    }
    
    /// Checks whether the view is complete (all measured)
    pub fn is_complete(&self) -> bool {
        self.observables.iter()
            .all(|obs| self.measured_values.contains_key(&obs.id))
    }
}

// ============================================================================
// VIEW MANAGER
// ============================================================================

/// Manager for multiple views
pub struct ViewManager {
    /// Managed views
    views: HashMap<Uuid, ObservableView>,
    
    /// Registered observables
    observables: HashMap<Uuid, Observable>,
    
    /// Statistics
    stats: ViewStatistics,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewStatistics {
    pub total_views_created: u64,
    pub total_observables_registered: u64,
    pub total_measurements: u64,
    pub avg_observables_per_view: f64,
}

impl ViewManager {
    pub fn new() -> Self {
        Self {
            views: HashMap::new(),
            observables: HashMap::new(),
            stats: ViewStatistics::default(),
        }
    }
    
    /// Registers an observable
    pub fn register_observable(&mut self, observable: Observable) -> Uuid {
        let id = observable.id;
        self.observables.insert(id, observable);
        self.stats.total_observables_registered += 1;
        id
    }
    
    /// Creates a new view
    pub fn create_view(&mut self, name: String, config: ViewConfig) -> Uuid {
        let view = ObservableView::new(name).with_config(config);
        let id = view.id;
        
        self.views.insert(id, view);
        self.stats.total_views_created += 1;
        
        id
    }
    
    /// Adds an observable to a view
    pub fn add_to_view(
        &mut self,
        view_id: Uuid,
        observable_id: Uuid,
    ) -> Result<(), String> {
        let view = self.views.get_mut(&view_id)
            .ok_or("View not found")?;
        
        let observable = self.observables.get(&observable_id)
            .ok_or("Observable not found")?
            .clone();
        
        view.add_observable(observable)
    }
    
    /// Measures view
    pub fn measure_view(
        &mut self,
        view_id: Uuid,
        shadow: &QuantumShadow,
    ) -> Result<(), String> {
        let view = self.views.get_mut(&view_id)
            .ok_or("View not found")?;
        
        view.measure_all(shadow)?;
        
        self.stats.total_measurements += view.observables.len() as u64;
        self.update_stats();
        
        Ok(())
    }
    
    /// Gets view
    pub fn get_view(&self, id: Uuid) -> Option<&ObservableView> {
        self.views.get(&id)
    }
    
    /// Lists all views
    pub fn list_views(&self) -> Vec<&ObservableView> {
        self.views.values().collect()
    }
    
    fn update_stats(&mut self) {
        if self.views.is_empty() {
            return;
        }
        
        let total_obs: usize = self.views.values()
            .map(|v| v.observables.len())
            .sum();
        
        self.stats.avg_observables_per_view = total_obs as f64 / self.views.len() as f64;
    }
    
    pub fn get_statistics(&self) -> &ViewStatistics {
        &self.stats
    }
}

impl Default for ViewManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// PREDEFINED OBSERVABLES
// ============================================================================

/// Factory for common observables
pub struct ObservableFactory;

impl ObservableFactory {
    /// Pauli-X on qubit i
    pub fn pauli_x(qubit: usize, num_qubits: usize) -> Observable {
        let mut paulis = vec![PauliOperator::I; num_qubits];
        paulis[qubit] = PauliOperator::X;
        
        Observable::from_pauli(
            format!("X_{}", qubit),
            PauliString::new(paulis),
        )
    }
    
    /// Pauli-Y on qubit i
    pub fn pauli_y(qubit: usize, num_qubits: usize) -> Observable {
        let mut paulis = vec![PauliOperator::I; num_qubits];
        paulis[qubit] = PauliOperator::Y;
        
        Observable::from_pauli(
            format!("Y_{}", qubit),
            PauliString::new(paulis),
        )
    }
    
    /// Pauli-Z on qubit i
    pub fn pauli_z(qubit: usize, num_qubits: usize) -> Observable {
        let mut paulis = vec![PauliOperator::I; num_qubits];
        paulis[qubit] = PauliOperator::Z;
        
        Observable::from_pauli(
            format!("Z_{}", qubit),
            PauliString::new(paulis),
        )
    }
    
    /// Hamiltonian de Ising ZZ
    pub fn ising_zz(qubit_a: usize, qubit_b: usize, num_qubits: usize) -> Observable {
        let mut paulis = vec![PauliOperator::I; num_qubits];
        paulis[qubit_a] = PauliOperator::Z;
        paulis[qubit_b] = PauliOperator::Z;
        
        let mut obs = Observable::from_pauli(
            format!("ZZ_{}_{}", qubit_a, qubit_b),
            PauliString::new(paulis),
        );
        obs.observable_type = ObservableType::Hamiltonian;
        obs
    }
    
    /// Stabilizer for surface code
    pub fn surface_code_stabilizer(
        qubits: Vec<usize>,
        num_qubits: usize,
        x_type: bool,
    ) -> Observable {
        let mut paulis = vec![PauliOperator::I; num_qubits];
        
        for &q in &qubits {
            paulis[q] = if x_type {
                PauliOperator::X
            } else {
                PauliOperator::Z
            };
        }
        
        let mut obs = Observable::from_pauli(
            format!("Stab_{:?}", qubits),
            PauliString::new(paulis),
        );
        obs.observable_type = ObservableType::Stabilizer;
        obs
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_observable_creation() {
        let obs = ObservableFactory::pauli_x(0, 2);
        
        assert_eq!(obs.name, "X_0");
        assert_eq!(obs.metadata.weight, 1);
        assert_eq!(obs.observable_type, ObservableType::Local);
    }
    
    #[test]
    fn test_commutativity() {
        let x0 = ObservableFactory::pauli_x(0, 2);
        let z0 = ObservableFactory::pauli_z(0, 2);
        let z1 = ObservableFactory::pauli_z(1, 2);
        
        // X and Z anticommute on the same qubit
        assert!(!x0.commutes_with(&z0));
        
        // X0 and Z1 commute (different qubits)
        assert!(x0.commutes_with(&z1));
    }
    
    #[test]
    fn test_view_creation() {
        let mut view = ObservableView::new("test_view".to_string());
        
        let obs = ObservableFactory::pauli_z(0, 2);
        view.add_observable(obs).unwrap();
        
        assert_eq!(view.observables.len(), 1);
    }
    
    #[test]
    fn test_commutativity_enforcement() {
        let mut view = ObservableView::new("test".to_string());
        
        let x0 = ObservableFactory::pauli_x(0, 2);
        let z0 = ObservableFactory::pauli_z(0, 2);
        
        view.add_observable(x0).unwrap();
        
        // Should fail because X and Z anticommute
        let result = view.add_observable(z0);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_view_manager() {
        let mut manager = ViewManager::new();
        
        // Register observables
        let obs_id = manager.register_observable(ObservableFactory::pauli_z(0, 2));
        
        // Create view
        let view_id = manager.create_view("test".to_string(), ViewConfig::default());
        
        // Add observable to the view
        manager.add_to_view(view_id, obs_id).unwrap();
        
        let view = manager.get_view(view_id).unwrap();
        assert_eq!(view.observables.len(), 1);
    }
    
    #[test]
    fn test_observable_factory() {
        let ising = ObservableFactory::ising_zz(0, 1, 3);
        
        assert_eq!(ising.observable_type, ObservableType::Hamiltonian);
        assert_eq!(ising.metadata.weight, 2);
    }
}
