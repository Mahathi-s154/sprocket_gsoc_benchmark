import time
import WDL  # miniwdl
import sprocket_py # Your Rust extension

# A standard WDL document for testing
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
"""

ITERATIONS = 5000

def benchmark_miniwdl():
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        try:
            # miniwdl parsing
            WDL.parse_document(WDL_DOC)
        except Exception:
            pass
    return time.perf_counter() - start

def benchmark_sprocket():
    start = time.perf_counter()
    for _ in range(ITERATIONS):
        try:
            # Your Rust FFI parsing
            sprocket_py.parse_ast(WDL_DOC)
        except Exception:
            pass
    return time.perf_counter() - start

print(f"Running Benchmark: Parsing {ITERATIONS} WDL documents...\n")

time_miniwdl = benchmark_miniwdl()
print(f"miniwdl (Pure Python): {time_miniwdl:.4f} seconds")

time_sprocket = benchmark_sprocket()
print(f"sprocket-py (Rust FFI):  {time_sprocket:.4f} seconds")

if time_sprocket < time_miniwdl:
    speedup = time_miniwdl / time_sprocket
    print(f"\n🚀 sprocket-py is {speedup:.1f}x faster!")