use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use static_analysis::{load_yara_rules, scan_bytes};

fn criterion_benchmark(c: &mut Criterion) {
    // ------------------------------------------------------------------
    // GROUP 1: Heavy Lifecycle Operations (Rule Loading)
    // ------------------------------------------------------------------
    let mut group_lifecycle = c.benchmark_group("lifecycle");
    
    // CRITICAL FIX: Reduce sample size. 
    // Loading rules is slow. We don't need to run it 100 times (default).
    // 10 samples is enough to get a statistical average without waiting forever.
    group_lifecycle.sample_size(10); 

    group_lifecycle.bench_function("load_rules", |b| {
        b.iter(|| {
            // pass the result to black_box so the compiler doesn't optimize 
            // the function call away if it thinks the result is unused.
            black_box(load_yara_rules(black_box("rules")))
        })
    });
    group_lifecycle.finish();

    // ------------------------------------------------------------------
    // GROUP 2: Scanning Throughput
    // ------------------------------------------------------------------
    
    // 1. Setup (Run once, outside the measurement loop)
    let rules = load_yara_rules("rules");
    
    // CRITICAL FIX: Don't use zeros. Use a repeating pattern or random noise.
    // Scanners often have "fast paths" for zero-buffers.
    // This creates a 1MB buffer filled with 0xAA (binary 10101010).
    let payload = vec![0xAAu8; 1024 * 1024]; 

    let mut group_scan = c.benchmark_group("scanning_throughput");
    
    // Set throughput so the output shows "MB/s"
    group_scan.throughput(Throughput::Bytes(payload.len() as u64));

    group_scan.bench_function("scan_1mb_buffer", |b| {
        b.iter(|| {
            // CRITICAL FIX: Pass the arguments through black_box AND
            // ensure the function actually runs by not letting the compiler verify the data is static.
            scan_bytes(
                black_box(&rules), 
                black_box(&payload)
            )
        })
    });

    group_scan.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);