from __future__ import annotations

import argparse
import gc
import platform
import sys
import time
from dataclasses import dataclass
from importlib.metadata import PackageNotFoundError, version
from statistics import mean, stdev

import WDL
import sprocket_py

WDL_DOC = """
version 1.0
workflow test_workflow {
    input { String name }
    call hello { input: name = name }
}
task hello {
    input { String name }
    command { echo "Hello ~{name}" }
    runtime { docker: "ubuntu:latest" }
}
""".strip()

DEFAULT_ITERATIONS = 5000
DEFAULT_ROUNDS = 5
DEFAULT_WARMUP_ROUNDS = 1


@dataclass(frozen=True)
class BenchmarkStats:
    name: str
    timings: list[float]
    iterations: int

    @property
    def average(self) -> float:
        return mean(self.timings)

    @property
    def fastest(self) -> float:
        return min(self.timings)

    @property
    def slowest(self) -> float:
        return max(self.timings)

    @property
    def sigma(self) -> float:
        return 0.0 if len(self.timings) < 2 else stdev(self.timings)

    @property
    def docs_per_second(self) -> float:
        return self.iterations / self.average


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Benchmark miniwdl against the Rust-backed sprocket_py parser."
    )
    parser.add_argument(
        "--iterations",
        type=int,
        default=DEFAULT_ITERATIONS,
        help=f"Number of parses per timing round (default: {DEFAULT_ITERATIONS}).",
    )
    parser.add_argument(
        "--rounds",
        type=int,
        default=DEFAULT_ROUNDS,
        help=f"Number of measured timing rounds (default: {DEFAULT_ROUNDS}).",
    )
    parser.add_argument(
        "--warmup-rounds",
        type=int,
        default=DEFAULT_WARMUP_ROUNDS,
        help=f"Number of warmup rounds before measurement (default: {DEFAULT_WARMUP_ROUNDS}).",
    )
    return parser.parse_args()


def package_version(name: str) -> str:
    try:
        return version(name)
    except PackageNotFoundError:
        return "unknown"


def validate_inputs() -> None:
    WDL.parse_document(WDL_DOC)

    diagnostic_count, diagnostics = sprocket_py.parse_ast(WDL_DOC)
    if diagnostic_count != 0:
        raise RuntimeError(
            "sprocket_py.parse_ast reported diagnostics for the benchmark input: "
            f"{diagnostics}"
        )


def run_parser(parser_fn, iterations: int) -> float:
    gc_enabled = gc.isenabled()
    if gc_enabled:
        gc.disable()

    try:
        start = time.perf_counter()
        for _ in range(iterations):
            parser_fn(WDL_DOC)
        return time.perf_counter() - start
    finally:
        if gc_enabled:
            gc.enable()


def benchmark(name: str, parser_fn, iterations: int, rounds: int, warmup_rounds: int) -> BenchmarkStats:
    for _ in range(warmup_rounds):
        run_parser(parser_fn, iterations)

    timings = [run_parser(parser_fn, iterations) for _ in range(rounds)]
    return BenchmarkStats(name=name, timings=timings, iterations=iterations)


def format_stats(stats: BenchmarkStats) -> str:
    return (
        f"{stats.name:<14}"
        f"{stats.average:>10.4f}"
        f"{stats.fastest:>10.4f}"
        f"{stats.slowest:>10.4f}"
        f"{stats.sigma:>10.4f}"
        f"{stats.docs_per_second:>14.2f}"
    )


def print_environment(iterations: int, rounds: int, warmup_rounds: int) -> None:
    print("Benchmark Environment")
    print(f"  Python       : {sys.version.split()[0]}")
    print(f"  Platform     : {platform.platform()}")
    print(f"  miniwdl      : {package_version('miniwdl')}")
    print(f"  sprocket_py  : {package_version('sprocket_py')}")
    print(f"  Iterations   : {iterations}")
    print(f"  Warmup rounds: {warmup_rounds}")
    print(f"  Measured     : {rounds}")
    print()


def print_summary(miniwdl_stats: BenchmarkStats, sprocket_stats: BenchmarkStats) -> None:
    print("Results")
    print(
        f"{'Parser':<14}"
        f"{'mean(s)':>10}"
        f"{'min(s)':>10}"
        f"{'max(s)':>10}"
        f"{'stdev':>10}"
        f"{'docs/sec':>14}"
    )
    print("-" * 68)
    print(format_stats(miniwdl_stats))
    print(format_stats(sprocket_stats))
    print()

    mean_speedup = miniwdl_stats.average / sprocket_stats.average
    best_speedup = miniwdl_stats.fastest / sprocket_stats.fastest
    print(f"Speedup vs miniwdl (mean): {mean_speedup:.2f}x")
    print(f"Speedup vs miniwdl (best): {best_speedup:.2f}x")


def main() -> None:
    args = parse_args()
    if args.iterations <= 0 or args.rounds <= 0 or args.warmup_rounds < 0:
        raise ValueError("iterations and rounds must be positive; warmup rounds cannot be negative")

    validate_inputs()
    print_environment(
        iterations=args.iterations,
        rounds=args.rounds,
        warmup_rounds=args.warmup_rounds,
    )

    miniwdl_stats = benchmark(
        name="miniwdl",
        parser_fn=WDL.parse_document,
        iterations=args.iterations,
        rounds=args.rounds,
        warmup_rounds=args.warmup_rounds,
    )
    sprocket_stats = benchmark(
        name="sprocket-py",
        parser_fn=sprocket_py.parse_ast,
        iterations=args.iterations,
        rounds=args.rounds,
        warmup_rounds=args.warmup_rounds,
    )
    print_summary(miniwdl_stats, sprocket_stats)


if __name__ == "__main__":
    main()
