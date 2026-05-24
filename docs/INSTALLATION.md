# Installation Guide - LightQOS

This guide covers the full installation of LightQOS on different platforms.

---

## 📋 System Requirements

### Minimum

- **OS**: Linux, macOS, or Windows (with WSL2)
- **Python**: 3.10 or higher
- **Rust**: 1.70 or higher
- **RAM**: 4 GB
- **Disk**: 2 GB free

### Recommended

- **OS**: Ubuntu 22.04 LTS or macOS 13+
- **Python**: 3.11+
- **Rust**: 1.75+
- **RAM**: 8 GB or more
- **Disk**: 5 GB free
- **GPU**: Optional (for AI)

---

## 🐧 Linux (Ubuntu/Debian)

### 1. Install System Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install Python 3.11
sudo apt install -y python3.11 python3.11-venv python3.11-dev

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install build dependencies
sudo apt install -y build-essential pkg-config libssl-dev
```

### 2. Install LightQOS

```bash
# Create virtual environment
python3.11 -m venv lightqos-env
source lightqos-env/bin/activate

# Install via pip
pip install --upgrade pip
pip install lightqos

# OR install from source
git clone https://github.com/yourusername/lightqos.git
cd lightqos
pip install -e ".[dev]"
```

### 3. Compile Rust Kernel

```bash
cd kernel
cargo build --release

# Verify compilation
cargo test
```

### 4. Verify Installation

```bash
# Test Python
python -c "import lightqos; print(lightqos.__version__)"

# Run tests
pytest tests/
```

---

## 🍎 macOS

### 1. Install Homebrew (if needed)

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

### 2. Install Dependencies

```bash
# Python 3.11
brew install python@3.11

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# OpenSSL
brew install openssl pkg-config
```

### 3. Install LightQOS

```bash
# Create virtual environment
python3.11 -m venv lightqos-env
source lightqos-env/bin/activate

# Install
pip install lightqos

# OR from source
git clone https://github.com/yourusername/lightqos.git
cd lightqos
pip install -e ".[dev]"
```

### 4. Compile Kernel

```bash
cd kernel
cargo build --release
cargo test
```

---

## 🪟 Windows (via WSL2)

### 1. Install WSL2

```powershell
# In PowerShell (as Admin)
wsl --install
wsl --set-default-version 2

# Restart computer
# Open WSL2 (Ubuntu)
```

### 2. Follow Linux Instructions

Inside WSL2, follow the Linux instructions above.

---

## 🐳 Docker

### Option 1: Pre-built Image

```bash
# Pull image
docker pull lightqos/lightqos:latest

# Run container
docker run -it --rm lightqos/lightqos:latest python

# Inside the container
>>> import lightqos
>>> print(lightqos.__version__)
```

### Option 2: Local Build

```bash
# Clone repository
git clone https://github.com/yourusername/lightqos.git
cd lightqos

# Build image
docker build -t lightqos:local .

# Run
docker run -it --rm lightqos:local bash
```

### Docker Compose (with quantum hardware)

```yaml
# docker-compose.yml
version: '3.8'

services:
  lightqos:
    image: lightqos/lightqos:latest
    environment:
      - IBM_QUANTUM_TOKEN=${IBM_QUANTUM_TOKEN}
      - IONQ_API_KEY=${IONQ_API_KEY}
    volumes:
      - ./examples:/app/examples
      - ./data:/app/data
    command: python examples/bell_state.py
```

```bash
# Run
docker-compose up
```

---

## 📦 Development Installation

To contribute to LightQOS:

### 1. Clone and Configure

```bash
# Fork on GitHub first, then:
git clone https://github.com/marciolscoutinho/lightqos.git
cd lightqos

# Add upstream
git remote add upstream https://github.com/original/lightqos.git

# Create virtual environment
python3.11 -m venv venv
source venv/bin/activate  # Linux/macOS
# or
venv\Scripts\activate  # Windows
```

### 2. Install with Dev Dependencies

```bash
# Install in editable mode with extras
pip install -e ".[dev,docs,test]"

# Install pre-commit hooks
pre-commit install
```

### 3. Compile Kernel in Debug Mode

```bash
cd kernel
cargo build  # Debug mode
cargo test
cargo clippy  # Linter
```

### 4. Configure IDE

#### VS Code

Install extensions:

- Python
- rust-analyzer
- Even Better TOML

Configure `.vscode/settings.json`:

```json
{
  "python.defaultInterpreterPath": "${workspaceFolder}/venv/bin/python",
  "python.linting.enabled": true,
  "python.linting.pylintEnabled": true,
  "python.formatting.provider": "black",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

#### PyCharm

1. File → Settings → Project Interpreter
2. Select `venv/bin/python`
3. Enable Rust plugin

---

## 🔧 Configuration

### Environment Variables

Create `.env` in the root directory:

```bash
# Quantum hardware APIs
IBM_QUANTUM_TOKEN=your_ibm_token_here
IONQ_API_KEY=your_ionq_key_here

# LightQOS configuration
LIGHTQOS_LOG_LEVEL=INFO
LIGHTQOS_CACHE_DIR=~/.lightqos/cache
LIGHTQOS_DEFAULT_BACKEND=simulator

# The Light (AI)
LIGHT_ENABLE_LEARNING=true
LIGHT_LOG_CONSCIOUSNESS=true
```

### Configuration File

Create `~/.lightqos/config.toml`:

```toml
[general]
log_level = "INFO"
cache_dir = "~/.lightqos/cache"
default_backend = "simulator"

[drivers.ibm]
api_url = "https://api.quantum-computing.ibm.com"
default_backend = "ibm_brisbane"
shots = 1024

[drivers.ionq]
api_url = "https://api.ionq.co/v0.3"
default_backend = "aria-1"
shots = 1024

[the_light]
enable_learning = true
log_consciousness = true
initial_level = "DORMANT"

[emf]
max_pairs = 10000
recycling_threshold = 0.5

[tlm]
default_optimization_level = 2
```

---

## 🧪 Installation Verification

### Verification Script

Create `verify_install.py`:

```python
#!/usr/bin/env python3
"""Verifies LightQOS installation"""

import sys

def verify_python():
    """Verifies Python version"""
    version = sys.version_info
    if version.major == 3 and version.minor >= 10:
        print(f"✅ Python {version.major}.{version.minor}.{version.micro}")
        return True
    else:
        print(f"❌ Python {version.major}.{version.minor} (requires 3.10+)")
        return False

def verify_lightqos():
    """Verifies LightQOS"""
    try:
        import lightqos
        print(f"✅ LightQOS {lightqos.__version__}")
        return True
    except ImportError as e:
        print(f"❌ LightQOS not found: {e}")
        return False

def verify_components():
    """Verifies components"""
    components = [
        ("protocols", "lightqos.protocols"),
        ("network", "lightqos.network"),
        ("the_light", "lightqos.the_light"),
    ]

    all_ok = True
    for name, module in components:
        try:
            __import__(module)
            print(f"✅ {name}")
        except ImportError:
            print(f"❌ {name}")
            all_ok = False

    return all_ok

def verify_drivers():
    """Verifies drivers"""
    from lightqos.drivers import SimulatorDriver

    try:
        driver = SimulatorDriver()
        print("✅ Simulator driver")
        return True
    except Exception as e:
        print(f"❌ Simulator driver: {e}")
        return False

def main():
    print("=== Verifying LightQOS Installation ===\n")

    checks = [
        verify_python(),
        verify_lightqos(),
        verify_components(),
        verify_drivers(),
    ]

    print("\n" + "="*40)
    if all(checks):
        print("✅ Installation OK!")
        return 0
    else:
        print("❌ Problems found")
        return 1

if __name__ == "__main__":
    sys.exit(main())
```

Run:

```bash
python verify_install.py
```

---

## ❓ Common Issues

### Error: "Rust compiler not found"

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Error: "OpenSSL headers missing"

```bash
# Ubuntu/Debian
sudo apt install libssl-dev

# macOS
brew install openssl
export PKG_CONFIG_PATH="/usr/local/opt/openssl/lib/pkgconfig"
```

### Error: "Python.h not found"

```bash
# Ubuntu/Debian
sudo apt install python3.11-dev

# macOS
brew reinstall python@3.11
```

### Rust kernel does not compile

```bash
# Clean cache
cd kernel
cargo clean

# Update Rust
rustup update

# Recompile
cargo build --release
```

---

## 🎓 Next Steps

After installation:

1. ✅ Read [Tutorials](tutorials/)
2. ✅ Run [Examples](../examples/)
3. ✅ Explore [API Reference](api/)
   
   

---

## 📞 Support

Having problems? Contact:

- **Issues**: [GitHub Issues](https://github.com/marciolscoutinho/lightqos/issues)
- **Email**: marciolscoutinho@gmail.com
