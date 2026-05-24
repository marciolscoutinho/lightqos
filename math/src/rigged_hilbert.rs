// ---------------------------------------------------------------------------
// LightQOS - Quantum Operating System
// rigged_hilbert.rs — Rigged Hilbert Spaces — Gel'fand triplets and Dirac bra-kets
//
// Copyright (c) 2021 - 2026 Márcio Coutinho
// Date: 21-10-2021
// All rights reserved.
// ---------------------------------------------------------------------------

use num_complex::Complex64;
use std::f64::consts::PI;

// ============================================================================
// Tipos base
// ============================================================================

/// Escalar complexo (alias de conveniência)
pub type C64 = Complex64;

/// Vetor de states num espaço de Hilbert finito-dimensional
pub type StateVec = Vec<C64>;

/// Matriz densidade (n×n complexa)
pub type DensityMatrix = Vec<Vec<C64>>;

// ============================================================================
// RiggedHilbertSpace
// ============================================================================

/// Espaço de Hilbert Equipado (Gel'fand Triplet: Φ ⊂ H ⊂ Φ')
///
/// Suporta states físicos regulares (em Φ) e states generalizados
/// como os eigenkets do operador posição |x⟩ ou momento |p⟩.
#[derive(Debug, Clone)]
pub struct RiggedHilbertSpace {
    /// Space dimension de Hilbert H (pode ser ∞ — representado por dimension de truncagem)
    pub dimension: usize,
    /// Índice de regularidade Schwartz (0 = L², k>0 = mais regular)
    pub regularity_index: u32,
    /// Norma do espaço (2 = norma L², ∞ = norma sup)
    pub norm_type: NormType,
}

/// Tipo de norma for o espaço
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NormType {
    L2,
    Sup,
    Sobolev(u32),
}

impl RiggedHilbertSpace {
    /// Creates um novo espaço de Hilbert equipado
    pub fn new(dimension: usize) -> Self {
        Self {
            dimension,
            regularity_index: 0,
            norm_type: NormType::L2,
        }
    }

    /// Creates o espaço de Schwartz S(ℝⁿ) — o núcleo Φ
    pub fn schwartz(dimension: usize, regularity: u32) -> Self {
        Self {
            dimension,
            regularity_index: regularity,
            norm_type: NormType::Sobolev(regularity),
        }
    }

    /// Verifies if um state é normalizado (⟨ψ|ψ⟩ = 1)
    pub fn is_normalized(&self, state: &StateVec) -> bool {
        let norm_sq: f64 = state.iter().map(|c| c.norm_sqr()).sum();
        (norm_sq - 1.0).abs() < 1e-10
    }

    /// Normaliza um state quantum
    pub fn normalize(&self, state: &StateVec) -> StateVec {
        let norm: f64 = state.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt();
        if norm < 1e-15 {
            return state.clone();
        }
        state.iter().map(|c| c / norm).collect()
    }

    /// Inner product ⟨φ|ψ⟩
    pub fn inner_product(&self, bra: &StateVec, ket: &StateVec) -> C64 {
        assert_eq!(bra.len(), ket.len(), "Dimensões incompatíveis");
        bra.iter().zip(ket.iter()).map(|(b, k)| b.conj() * k).sum()
    }

    /// L² norm de um state
    pub fn l2_norm(&self, state: &StateVec) -> f64 {
        self.inner_product(state, state).re.sqrt()
    }

    /// Produto tensorial de dois states (|ψ⟩ ⊗ |φ⟩)
    pub fn tensor_product(&self, psi: &StateVec, phi: &StateVec) -> StateVec {
        let mut result = Vec::with_capacity(psi.len() * phi.len());
        for &a in psi.iter() {
            for &b in phi.iter() {
                result.push(a * b);
            }
        }
        result
    }

    /// Projector |ψ⟩⟨ψ| (densidade de state puro)
    pub fn projector(&self, state: &StateVec) -> DensityMatrix {
        let n = state.len();
        let mut mat = vec![vec![C64::new(0.0, 0.0); n]; n];
        for i in 0..n {
            for j in 0..n {
                mat[i][j] = state[i] * state[j].conj();
            }
        }
        mat
    }

    /// Partial trace (tracing out system B de um state bipartido AB)
    ///
    /// `dim_a` × `dim_b` = total dimension
    pub fn partial_trace_b(&self, rho: &DensityMatrix, dim_a: usize, dim_b: usize) -> DensityMatrix {
        let mut rho_a = vec![vec![C64::new(0.0, 0.0); dim_a]; dim_a];
        for i in 0..dim_a {
            for j in 0..dim_a {
                for k in 0..dim_b {
                    rho_a[i][j] += rho[i * dim_b + k][j * dim_b + k];
                }
            }
        }
        rho_a
    }

    /// Von Neumann entropy S(ρ) = -Tr(ρ log ρ)
    /// (implementation aproximada via diagonal da matriz densidade)
    pub fn von_neumann_entropy(&self, rho: &DensityMatrix) -> f64 {
        // Usa eigenvalues aproximados via diagonal (válido for states quase-diagonais)
        let n = rho.len();
        let mut entropy = 0.0;
        for i in 0..n {
            let lambda = rho[i][i].re.max(0.0);
            if lambda > 1e-15 {
                entropy -= lambda * lambda.ln();
            }
        }
        entropy
    }

    /// Fidelidade entre dois states puros F = |⟨φ|ψ⟩|²
    pub fn fidelity_pure(&self, psi: &StateVec, phi: &StateVec) -> f64 {
        self.inner_product(phi, psi).norm_sqr()
    }

    /// Fidelidade de Uhlmann entre states mistos (aproximação via sqrt)
    pub fn fidelity_mixed(&self, rho: &DensityMatrix, sigma: &DensityMatrix) -> f64 {
        let n = rho.len();
        // Aproximação: F ≈ Tr(ρσ) for states próximos
        let mut trace = C64::new(0.0, 0.0);
        for i in 0..n {
            for j in 0..n {
                trace += rho[i][j] * sigma[j][i];
            }
        }
        trace.re.max(0.0).min(1.0)
    }
}

// ============================================================================
// DiracState — Ket e Bra no formalismo de Dirac
// ============================================================================

/// Dirac state in the rigged Hilbert space
///
/// Represents both regular kets |ψ⟩ ∈ Φ como funcionais
/// generalizadas ⟨x| ∈ Φ' (distribuições de Dirac).
#[derive(Debug, Clone)]
pub struct DiracState {
    pub label: String,
    pub components: StateVec,
    pub is_generalized: bool,
}

impl DiracState {
    /// Creates um ket normalizado |ψ⟩
    pub fn ket(label: impl Into<String>, components: Vec<C64>) -> Self {
        let space = RiggedHilbertSpace::new(components.len());
        let components = space.normalize(&components);
        Self { label: label.into(), components, is_generalized: false }
    }

    /// Computational basis state |n⟩
    pub fn basis(n: usize, dim: usize) -> Self {
        let mut v = vec![C64::new(0.0, 0.0); dim];
        if n < dim { v[n] = C64::new(1.0, 0.0); }
        Self {
            label: format!("|{}>", n),
            components: v,
            is_generalized: false,
        }
    }

    /// Estado de Bell |Φ+⟩ = (|00⟩ + |11⟩)/√2
    pub fn bell_phi_plus() -> Self {
        let s = 1.0 / 2.0_f64.sqrt();
        Self {
            label: "|Φ+>".into(),
            components: vec![
                C64::new(s, 0.0),
                C64::new(0.0, 0.0),
                C64::new(0.0, 0.0),
                C64::new(s, 0.0),
            ],
            is_generalized: false,
        }
    }

    /// Estado de Bell |Ψ-⟩ = (|01⟩ - |10⟩)/√2 (singlete)
    pub fn bell_psi_minus() -> Self {
        let s = 1.0 / 2.0_f64.sqrt();
        Self {
            label: "|Ψ->".into(),
            components: vec![
                C64::new(0.0, 0.0),
                C64::new(s, 0.0),
                C64::new(-s, 0.0),
                C64::new(0.0, 0.0),
            ],
            is_generalized: false,
        }
    }

    /// Estado de superposição uniforme |+⟩^⊗n (Hadamard em all os qubits)
    pub fn uniform_superposition(n_qubits: usize) -> Self {
        let dim = 1 << n_qubits;
        let amp = C64::new(1.0 / (dim as f64).sqrt(), 0.0);
        Self {
            label: format!("|+>^{}", n_qubits),
            components: vec![amp; dim],
            is_generalized: false,
        }
    }

    /// Space dimension
    pub fn dim(&self) -> usize { self.components.len() }

    /// L² norm
    pub fn norm(&self) -> f64 {
        self.components.iter().map(|c| c.norm_sqr()).sum::<f64>().sqrt()
    }
}

// ============================================================================
// GeneralizedEigenfunction — autofunção generalizada
// ============================================================================

/// Autofunção generalizada de um operador (for eigenvalues contínuos)
///
/// Ex: eigenfunções do operador posição Q|x⟩ = x|x⟩
/// ou do operador momento P|p⟩ = p|p⟩
#[derive(Debug, Clone)]
pub struct GeneralizedEigenfunction {
    pub eigenvalue: f64,
    pub operator_name: String,
    /// Amostras da function de onda em points discretos da grade
    pub samples: Vec<C64>,
    pub grid_points: Vec<f64>,
}

impl GeneralizedEigenfunction {
    /// Eigenfunction do operador momento P: ψ_p(x) = e^(ipx)/√(2π)
    pub fn momentum_eigenfunction(p: f64, x_grid: &[f64]) -> Self {
        let samples: Vec<C64> = x_grid.iter()
            .map(|&x| C64::new(0.0, p * x).exp() / (2.0 * PI).sqrt())
            .collect();
        Self {
            eigenvalue: p,
            operator_name: "P".into(),
            samples,
            grid_points: x_grid.to_vec(),
        }
    }

    /// Eigenfunction do hamiltoniano harmónico H_n(x) (polinómio de Hermite)
    pub fn harmonic_oscillator(n: usize, x_grid: &[f64]) -> Self {
        let samples: Vec<C64> = x_grid.iter()
            .map(|&x| {
                let h = hermite_polynomial(n, x);
                let psi = (-x * x / 2.0).exp() * h
                    / (2.0_f64.powi(n as i32)
                        * factorial(n) as f64
                        * PI.sqrt())
                    .sqrt();
                C64::new(psi, 0.0)
            })
            .collect();
        let energy = (n as f64) + 0.5; // ℏω = 1
        Self {
            eigenvalue: energy,
            operator_name: format!("H_ho(n={})", n),
            samples,
            grid_points: x_grid.to_vec(),
        }
    }
}

// Funções auxiliares
fn hermite_polynomial(n: usize, x: f64) -> f64 {
    match n {
        0 => 1.0,
        1 => 2.0 * x,
        _ => {
            let mut h_prev = 1.0;
            let mut h_curr = 2.0 * x;
            for k in 1..n {
                let h_next = 2.0 * x * h_curr - 2.0 * k as f64 * h_prev;
                h_prev = h_curr;
                h_curr = h_next;
            }
            h_curr
        }
    }
}

fn factorial(n: usize) -> u64 {
    (1..=n as u64).product::<u64>().max(1)
}

// ============================================================================
// TESTES
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn test_normalization() {
        let space = RiggedHilbertSpace::new(2);
        let state = vec![C64::new(3.0, 0.0), C64::new(4.0, 0.0)];
        let normalized = space.normalize(&state);
        assert!(space.is_normalized(&normalized));
    }

    #[test]
    fn test_inner_product_orthogonal() {
        let space = RiggedHilbertSpace::new(2);
        let zero = DiracState::basis(0, 2);
        let one = DiracState::basis(1, 2);
        let ip = space.inner_product(&zero.components, &one.components);
        assert_abs_diff_eq!(ip.re, 0.0, epsilon = 1e-12);
        assert_abs_diff_eq!(ip.im, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_bell_state_normalization() {
        let bell = DiracState::bell_phi_plus();
        assert_abs_diff_eq!(bell.norm(), 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_uniform_superposition() {
        let sup = DiracState::uniform_superposition(3);
        assert_eq!(sup.dim(), 8);
        assert_abs_diff_eq!(sup.norm(), 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_von_neumann_entropy_pure_state() {
        let space = RiggedHilbertSpace::new(2);
        let state = DiracState::basis(0, 2);
        let rho = space.projector(&state.components);
        // Estado puro → entropia = 0
        let s = space.von_neumann_entropy(&rho);
        assert_abs_diff_eq!(s, 0.0, epsilon = 1e-12);
    }

    #[test]
    fn test_fidelity_identical_states() {
        let space = RiggedHilbertSpace::new(2);
        let psi = DiracState::bell_phi_plus();
        let f = space.fidelity_pure(&psi.components, &psi.components);
        assert_abs_diff_eq!(f, 1.0, epsilon = 1e-12);
    }

    #[test]
    fn test_momentum_eigenfunction() {
        let x_grid: Vec<f64> = (0..10).map(|i| i as f64 * 0.1).collect();
        let ef = GeneralizedEigenfunction::momentum_eigenfunction(1.0, &x_grid);
        assert_eq!(ef.samples.len(), 10);
        assert_eq!(ef.operator_name, "P");
    }

    #[test]
    fn test_hermite_polynomials() {
        // H_0(x) = 1, H_1(x) = 2x, H_2(x) = 4x²-2
        assert_abs_diff_eq!(hermite_polynomial(0, 1.0), 1.0, epsilon = 1e-12);
        assert_abs_diff_eq!(hermite_polynomial(1, 1.0), 2.0, epsilon = 1e-12);
        assert_abs_diff_eq!(hermite_polynomial(2, 1.0), 2.0, epsilon = 1e-12);
    }
}
