# Contributing Guide - LightQOS

Thank you for considering contributing to LightQOS! 🎉

---

## 📋 How to Contribute

### 1. Fork and Clone

```bash
# Fork on GitHub
# Then clone your fork
git clone https://github.com/marciolscoutinho/lightqos.git
cd lightqos

# Add upstream
git remote add upstream https://github.com/marciolscoutinho/lightqos.git
```

### 2. Create a Branch

```bash
# Update main
git checkout main
git pull upstream main

# Create a branch for your feature
git checkout -b feature/feature-name
```

### 3. Make Changes

```bash
# Install in development mode
pip install -e ".[dev]"

# Make changes
# Write tests
# Document your changes

# Run tests
pytest tests/

# Check code quality
black lightqos/
pylint lightqos/
```

### 4. Commit

```bash
# Commit with a descriptive message
git add .
git commit -m "feat: add new feature X"

# Follow Conventional Commits:
# feat: new feature
# fix: bug fix
# docs: documentation
# test: tests
# refactor: refactoring
# style: formatting
```

### 5. Push and Pull Request

```bash
# Push to your fork
git push origin feature/feature-name

# Open a PR on GitHub
# Fill in the PR template
```

---

## 🎯 Contribution Areas

### 🐛 Report Bugs

Open an [issue](https://github.com/yourusername/lightqos/issues) with:

- A clear description of the bug
- Steps to reproduce
- Expected behavior vs actual behavior
- LightQOS version
- OS and Python/Rust version

### ✨ Propose Features

Open a [feature request issue](https://github.com/yourusername/lightqos/issues/new?template=feature_request.md) with:

- Feature description
- Rationale: why is it useful?
- Usage example
- Possible implementation

### 📝 Improve Documentation

- Fix typos
- Add examples
- Write tutorials
- Translate documentation

### 🧪 Add Tests

- Increase coverage
- Integration tests
- Benchmarks
- Real hardware tests

### 🔧 Develop Drivers

New drivers for quantum hardware:

- Rigetti
- Google Cirq
- AWS Braket
- Azure Quantum

---

## 📏 Code Standards

### Python

```python
# Follow PEP 8
# Use Black for formatting
black lightqos/

# Type hints are mandatory
def function_name(arg: int) -> str:
    """Docstring with a description.

    Args:
        arg: Argument description

    Returns:
        Return value description
    """
    return str(arg)

# Use Google-style docstrings
```

### Rust

```rust
// Follow Rust guidelines
// Use clippy
cargo clippy

// Documentation is mandatory
/// Description of function
///
/// # Arguments
///
/// * `arg` - Description of arg
///
/// # Returns
///
/// Description of return
pub fn function_name(arg: i32) -> String {
    format!("{}", arg)
}
```

---

## 🧪 Tests

### Writing Tests

```python
# tests/test_my_feature.py
import pytest
from lightqos import MyFeature

def test_my_feature():
    """Test description"""
    feature = MyFeature()
    result = feature.do_something()
    assert result == expected

@pytest.mark.asyncio
async def test_async_feature():
    """Test async feature"""
    result = await async_function()
    assert result is not None
```

### Running Tests

```bash
# All tests
pytest tests/

# Specific test file
pytest tests/test_my_feature.py

# With coverage
pytest --cov=lightqos tests/

# By specific marker
pytest -m "slow"      # Slow tests
pytest -m "not slow"  # Exclude slow tests
```

---

## 📚 Documentation

### Docstrings

```python
def teleport(state: QuantumState) -> TeleportationResult:
    """
    Executes quantum teleportation.

    Args:
        state: Quantum state to teleport

    Returns:
        Result containing fidelity and measured bits

    Raises:
        ValueError: If the state is invalid

    Example:
        >>> from lightqos.protocols import QuantumTeleportation
        >>> protocol = QuantumTeleportation()
        >>> result = protocol.teleport(state)
        >>> print(f"Fidelity: {result.final_fidelity}")
    """
```

### Markdown

- Use appropriate headers
- Use code blocks with language identifiers
- Use working links
- Store images in `docs/images/`

---

## 🔍 Code Review

Pull Requests will be reviewed for:

- **Functionality**: Does the code work?
- **Tests**: Is coverage adequate?
- **Documentation**: Is it well documented?
- **Style**: Does it follow the standards?
- **Performance**: Is it efficient?

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

## 🙏 Acknowledgements

All contributors are listed in [CONTRIBUTORS.md](CONTRIBUTORS.md)!

---

## ❓ Questions?

- Email: marciolscoutinho@gmail.com
