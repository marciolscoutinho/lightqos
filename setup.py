#!/usr/bin/env python3
# -----------------------------------------------------------------------------
# LightQOS - Quantum Operating System
# setup.py — Python package setup script (legacy pip compatibility)
#
# Copyright (c) 2021 - 2026 Márcio Coutinho
# Date: 21-03-2022
# All rights reserved.
# -----------------------------------------------------------------------------

"""
Setup script for LightQOS.

Compiles the Rust kernel and creates Python bindings via PyO3/maturin.
Preferred usage: maturin develop / maturin build.
This setup.py file exists for compatibility with pip install -e .
"""

from setuptools import setup, find_packages

# Read README
with open("README.md", "r", encoding="utf-8") as fh:
    long_description = fh.read()

setup(
    name="lightqos",
    version="0.2.0",
    author="Márcio Coutinho",
    author_email="marciolscoutinho@gmail.com",
    description="Light Quantum Operating System — Quantum OS with The Light AI",
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/marciolscoutinho/lightqos",
    project_urls={
        "Bug Tracker": "https://github.com/marciolscoutinho/lightqos/issues",
        "Documentation": "https://github.com/marciolscoutinho/lightqos/tree/main/docs",
        "Source Code": "https://github.com/marciolscoutinho/lightqos",
    },
    package_dir={"": "frontend"},
    packages=find_packages(where="frontend"),
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Rust",
        "Topic :: Scientific/Engineering :: Physics",
    ],
    python_requires=">=3.10",
    install_requires=[
        "numpy>=1.24.0",
        "scipy>=1.11.0",
        "pydantic>=2.0.0",
        "rich>=13.0.0",
        "click>=8.1.0",
    ],
    extras_require={
        "ai":       ["torch>=2.0.0", "transformers>=4.35.0"],
        "qiskit":   ["qiskit>=0.44.0"],
        "cirq":     ["cirq>=1.2.0"],
        "pennylane":["pennylane>=0.33.0"],
        "dev":      ["pytest>=7.4", "pytest-cov>=4.1", "mypy>=1.5", "ruff>=0.1"],
        "all":      ["torch>=2.0", "qiskit>=0.44", "cirq>=1.2", "pennylane>=0.33",
                     "pytest>=7.4", "ruff>=0.1"],
    },
)
