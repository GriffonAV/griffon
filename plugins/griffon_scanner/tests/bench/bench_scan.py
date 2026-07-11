import argparse
import subprocess
import json

parser = argparse.ArgumentParser()
parser.add_argument("--binary", required=True, help="Path to compiled Rust bench_runner")
parser.add_argument("--target", required=True, help="Directory to scan")
parser.add_argument("--sweep-threads", required=True, help="Comma separated thread counts (e.g. 1,2,4,auto)")
parser.add_argument("--iterations", type=int, default=3, help="How many times to run each test")
args = parser.parse_args()

threads = args.sweep_threads.split(",")

print("\nStarting Griffon Performance Benchmark")
print(f"Target: {args.target} | Iterations per thread count: {args.iterations}\n")
print(f"{'Threads':<12} | {'Avg Time (sec)':<15} | {'Speedup':<10}")
print("-" * 45)

base_time = None

for t in threads:
    times = []
    for _ in range(args.iterations):
        cmd = [args.binary, "--target", args.target, "--threads", t]
        result = subprocess.run(cmd, capture_output=True, text=True)
        
        try:
            data = json.loads(result.stdout)
            times.append(data["time_taken"])
        except json.JSONDecodeError:
            print(f"Error parsing Rust output. Raw output:\n{result.stdout}")
            exit(1)
            
    avg_time = sum(times) / len(times)
    
    if base_time is None:
        base_time = avg_time
        speedup = "1.00x"
    else:
        speedup = f"{(base_time / avg_time):.2f}x"
        
    print(f"{t:<12} | {avg_time:<15.4f} | {speedup:<10}")
print("-" * 45)