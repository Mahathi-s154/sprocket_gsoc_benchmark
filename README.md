# sprocket-py-prototype (GSoC 2026 Proof of Concept)

> A minimal PyO3/Maturin bridge that exposes St. Jude Cloud's Rust-based WDL parser to Python for significantly faster parsing in bioinformatics workflows.

This repository is a proof of concept for **Google Summer of Code 2026** around Python bindings for Sprocket's WDL tooling. The core parser lives in the Rust workspace maintained by the St. Jude Cloud team in [`stjude-rust-labs/wdl`](https://github.com/stjude-rust-labs/wdl), while many downstream bioinformatics users primarily work in Python.  
The goal of this prototype is to show that a lightweight Rust extension can preserve the performance characteristics of the native parser while presenting a Python-friendly entry point.

## Why This Prototype Matters

WDL parsing is a foundational operation for workflow validation, tooling, IDE support, and pipeline orchestration. In Python-centric environments, parser throughput can become a bottleneck. This prototype demonstrates a direct path to:

- reuse the existing Rust parser implementation instead of reimplementing parsing logic in Python,
- expose parser functionality through a familiar Python import,
- create a foundation for future high-level Python models built on top of Sprocket's Rust AST.

## Implemented Features

The current proof of concept intentionally keeps the Python API small and verifiable:

| Python API | Rust Entry Point | Current Return Value |
| --- | --- | --- |
| `parse_ast(source: str)` | `wdl_ast::Document::parse` | `(diagnostic_count, diagnostics)` |
| `parse_cst(source: str)` | `wdl_grammar::SyntaxTree::parse` | `(diagnostic_count, diagnostics)` |

- `parse_ast(source: &str)` maps directly to `wdl_ast::Document::parse`.
- `parse_cst(source: &str)` maps directly to `wdl_grammar::SyntaxTree::parse`.
- Both functions currently return a tuple of diagnostic count and rendered diagnostics.
- For valid WDL input, the expected result is `(0, [])`.

## Architecture At a Glance

```text
Python code
    |
    v
PyO3 extension module (sprocket_py)
    |
    +--> wdl_ast::Document::parse(...)     -> typed AST path
    |
    +--> wdl_grammar::SyntaxTree::parse(...) -> CST / grammar path
```

## Repository Layout

```text
.
├── benchmark.py
├── README.md
└── sprocket_py/
    ├── Cargo.toml
    ├── pyproject.toml
    └── src/lib.rs
```

## Build Instructions

### Prerequisites

- Python 3.8+
- Rust toolchain (`rustup`, `cargo`, `rustc`)

### Local Setup

```bash
git clone <your-repo-url>
cd sprocket-gsoc-prototype

python3 -m venv .venv
source .venv/bin/activate

python -m pip install --upgrade pip
python -m pip install maturin miniwdl

cd sprocket_py
maturin develop
```

After `maturin develop`, the extension is installed into the active virtual environment in editable mode.

## Python Smoke Test

```python
import sprocket_py
source = "version 1.1\nworkflow hello {}"
print(sprocket_py.parse_ast(source))
```

Expected output for valid input:

```python
(0, [])
```

## Benchmark Results

This repository includes a benchmark harness in [`benchmark.py`](./benchmark.py) to compare the Rust-backed binding against `miniwdl`, a widely used pure-Python baseline. The script validates both parsers on the same WDL input, runs a warmup pass, and reports multi-round summary statistics.

Run it with:

```bash
source .venv/bin/activate
python benchmark.py
```

Optional flags:

```bash
python benchmark.py --iterations 5000 --rounds 5 --warmup-rounds 1
```

### Results

Measured on March 27, 2026 in the local project virtual environment with Python `3.12.3`, `miniwdl 1.13.1`, `sprocket_py 0.1.0`, `5000` parses per round, `1` warmup round, and `5` measured rounds.

| Parser | Mean Time | Fastest | Slowest | Std. Dev. | Throughput |
| --- | --- | --- | --- | --- | --- |
| `miniwdl` | `2.1070 s` | `2.0523 s` | `2.1592 s` | `0.0493 s` | `2373.08 docs/sec` |
| `sprocket-py` | `0.3864 s` | `0.3854 s` | `0.3881 s` | `0.0010 s` | `12940.47 docs/sec` |

**Result:** `sprocket-py` was `5.45x` faster than `miniwdl` on mean runtime for this benchmark input.

## Next Steps for GSoC

This proof of concept focuses on validating the FFI boundary and confirming that the Rust parser can be invoked cleanly from Python. The longer-term GSoC objective is to move beyond raw diagnostic tuples and expose the full `wdl_ast` as rich Python-native objects.

Planned directions include:

- mapping the Rust AST into Python `@dataclass` or `Pydantic` models,
- exposing structured diagnostics, spans, and node metadata,
- designing a native-feeling Python API for downstream bioinformatics tooling,
- preserving Rust-side performance while improving Python developer ergonomics.

## Current Status

This prototype demonstrates that:

- the Rust parser can be compiled into a Python extension with PyO3 and Maturin,
- both AST-level and CST-level parsing paths are callable from Python,
- valid WDL input successfully returns zero diagnostics,
- the foundation is in place for a more complete Python binding layer during GSoC 2026.
