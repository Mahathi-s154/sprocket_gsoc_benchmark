# sprocket-py-prototype (GSoC 2026 Proof of Concept)

> A PyO3/Maturin proof of concept showing that Sprocket's Rust WDL parser can power a Python API with structured diagnostics while still preserving a fast benchmark path.

This repository is a proof of concept for **Google Summer of Code 2026** around Python bindings for Sprocket's WDL tooling. The parser itself lives in the Rust workspace maintained by the St. Jude Cloud team in [`stjude-rust-labs/wdl`](https://github.com/stjude-rust-labs/wdl). This prototype answers two questions:

1. Can the Rust parser be exposed cleanly to Python?
2. Can the first Python-facing types be designed in a way that is useful for downstream tooling without committing to full AST exposure too early?

The answer to both is yes.

## What This PoC Demonstrates

- a Rust parser can be packaged as a Python extension with PyO3 and Maturin,
- the binding can expose **structured diagnostics** instead of raw debug strings,
- the original high-throughput benchmark path can be preserved for apples-to-apples performance measurement,
- Python-facing API tests can validate the extension at the boundary where downstream users actually interact with it.

## Current Python API

The prototype now exposes both a structured API and legacy compatibility wrappers.

| Python API | Rust Entry Point | Return Value | Purpose |
| --- | --- | --- | --- |
| `parse(source: str)` | `wdl_ast::Document::parse` | `ParseResult` | Primary structured AST-backed parse path |
| `parse_cst_structured(source: str)` | `wdl_grammar::SyntaxTree::parse` | `ParseResult` | Primary structured CST-backed parse path |
| `parse_ast(source: str)` | `wdl_ast::Document::parse` | `(diagnostic_count, diagnostics)` | Legacy benchmark-compatible wrapper |
| `parse_cst(source: str)` | `wdl_grammar::SyntaxTree::parse` | `(diagnostic_count, diagnostics)` | Legacy benchmark-compatible wrapper |

### Structured types

The structured surface is intentionally small:

- `ParseResult`
  - `diagnostics: list[PyDiagnostic]`
  - `version: str | None`
  - `document_kind: str | None`
- `PyDiagnostic`
  - `rule: str | None`
  - `severity: str`
  - `message: str`
  - `fix: str | None`
  - `labels: list[PyLabel]`
- `PyLabel`
  - `message: str`
  - `span: PySpan`
- `PySpan`
  - `start: int`
  - `end: int`

`document_kind` is currently left as `None`. This PoC avoids guessing at a stable Python contract for higher-level AST categorization before the larger binding design is settled.

## What We Learned From The Exploration

### Valuable first Python-facing types

- **Diagnostic** is worth exposing because it is immediately useful to CLI tooling, validation pipelines, editors, and tests.
- **Label** and **Span** matter because diagnostics are much less useful without source ranges.
- **Version** is a good top-level result field because Python callers often need basic document metadata without traversing a full AST.

### Types that should stay internal for now

- Generic AST/CST node graphs should remain Rust-internal at this stage.
- Raw syntax nodes and token machinery are too implementation-shaped for a first Python API.
- Exposing the full AST before agreeing on Python ergonomics would create a large, unstable surface too early.

### Hard problem discovered but deferred

The next major technical proof is **async analysis bridging**. The `wdl-analysis` layer is asynchronous, so a future PoC should show:

- runtime management from Python,
- GIL release around blocking Rust work,
- a minimal `lint()` or analysis-oriented entry point,
- error mapping across the async boundary.

That work is intentionally deferred from this revision so the current PoC stays focused on structured type exposure and Python API testing.

## Architecture At A Glance

```text
Python code
    |
    v
PyO3 extension module (sprocket_py)
    |
    +--> parse(...)                -> structured AST-backed ParseResult
    +--> parse_cst_structured(...) -> structured CST-backed ParseResult
    +--> parse_ast(...)            -> legacy tuple wrapper for benchmarks
    +--> parse_cst(...)            -> legacy tuple wrapper for benchmarks
```

## Repository Layout

```text
.
├── .venv/
├── benchmark.py
├── README.md
└── sprocket_py/
    ├── Cargo.toml
    ├── pyproject.toml
    ├── src/
    │   └── lib.rs
    └── tests/
        └── test_sprocket_py.py
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
python -m pip install maturin miniwdl pytest

cd sprocket_py
maturin develop
```

After `maturin develop`, the extension is installed into the active virtual environment in editable mode.

## Python Smoke Test

```python
import sprocket_py

result = sprocket_py.parse("version 1.1\nworkflow hello {}")
print(result.version)
print(len(result.diagnostics))
```

Expected output:

```python
1.1
0
```

The benchmark-compatible wrapper is still available:

```python
import sprocket_py

print(sprocket_py.parse_ast("version 1.1\nworkflow hello {}"))
```

Expected output for valid input:

```python
(0, [])
```

## Tests

Python-facing tests live in `sprocket_py/tests/` and exercise the installed extension module rather than only Rust internals.

Run them with:

```bash
source .venv/bin/activate
cd sprocket_py
pytest
```

Rust unit tests remain available as crate-level smoke coverage:

```bash
cd sprocket_py
cargo test
```

## Benchmark Results

This repository includes a benchmark harness in [`benchmark.py`](./benchmark.py) to compare the Rust-backed binding against `miniwdl`, a widely used pure-Python baseline. The script validates both parsers on the same WDL input, runs a warmup pass, and reports multi-round summary statistics.

The benchmark continues to use the legacy `parse_ast()` compatibility wrapper so the structured Python API does not distort the original performance comparison.

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

## What This PoC Still Does Not Deliver

This repository is still a proof of concept, not the final GSoC package. It does **not** yet provide:

- full Python-native AST models,
- async analysis or linting entry points,
- rich Python exceptions mapped from analysis/runtime failures,
- a finalized public API for workflow/task/document introspection.

Those are the areas a larger `sprocket-py` effort should tackle next.
