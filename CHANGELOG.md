# Changelog

All notable changes to this project are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versions follow [Semantic Versioning](https://semver.org/).

---

## [Unreleased]

### Planned

- Photonic driver (Xanadu PennyLane-SF)
- Web dashboard for real-time EMF monitoring
- Integration with IBM Qiskit Runtime v2
- The Light AI — public pre-trained model on HuggingFace

---

## [0.2.0] — 2026-05-24

### Added

- **Drivers**: QbloxDriver (QCM/QRM) and ZurichDriver (SHFQA/SHFSG/HDAWG)
- **HIO**: `py_hio.rs` module — complete PyO3 bindings for Shadow Tomography
- **TLM**: `py_tlm.rs` module — ContractManager and HarmonicScheduler via PyO3
- **Math**: `rigged_hilbert.rs` — Rigged Hilbert Spaces (Gel'fand Triplets)
- **Docs**: Complete documentation (architecture, Grover/QFT/VQE tutorials, API references)
- **CI/CD**: GitHub Actions (ci.yml + release.yml) with a multi-platform build matrix
- **Examples**: `bell_state.py`, `qkd_demo.py`, `the_light_showcase.py`
- **Tests**: `test_pyo3_integration.py`, `test_protocols.py`, `benchmark_performance.py`
- Unified Cargo.toml workspace with 8 crates
- `pyproject.toml` with full maturin support

### Changed

- `setup.py` updated to v0.2.0 with the correct GitHub URL
- `README.md` fully rewritten with tables and badges
- `drivers/src/mod.rs` — Qblox and Zurich re-exports
- `math/src/lib.rs` — complete re-exports

### Fixed

- Duplicate files between LightQOS v0.1.0 and LightQOS v0.1.1 resolved
- Old `mod2.rs` files removed
- GitHub URLs standardized to `marciolscoutinho`

---

## [0.1.0] — 2021-12-01

### Added

- Base architecture: EFAL, EMF, TLM, HIO
- Drivers: IBM Quantum, IonQ, Simulator
- Python frontend: `QuantumCircuit`, `TemporalContract`
- Integrations: Qiskit, Cirq, PennyLane
- The Light AI: `TranspilerOptimizer`, `EMFPredictor`, `ConsciousnessMath`
- Process Tensor Framework (Quantum Combs)
- Advanced Shadow Tomography (adaptive resampling, mid-circuit feedback)
- Protocols: T-HQC, QCR, QLC, ZPE Extraction
- Basic CLI
