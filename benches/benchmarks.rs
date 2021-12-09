use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use rolling_dual_crc::{DualCrc, RollingDualCrc, Zeros};

fn benchmarks(c: &mut Criterion) {
    // ============================================================
    // MAIN

    let bytes_1k = vec![b'x'; 1024];
    let bytes_32k = vec![b'x'; 32 * 1024];
    let bytes_1m = vec![b'x'; 1024 * 1024];
    let mut crc = DualCrc::new();
    let mut crc_1k = RollingDualCrc::new(&bytes_1k);
    let mut crc_32k = RollingDualCrc::new(&bytes_32k);
    let mut crc_1m = RollingDualCrc::new(&bytes_1m);

    let mut group = c.benchmark_group("main");

    group.throughput(Throughput::Bytes(1024));

    group.bench_function("DualCrc::checksum 1 kiB", |b| {
        b.iter(|| DualCrc::checksum(black_box(&bytes_1k)))
    });

    group.bench_function("DualCrc::checksum32 1 kiB", |b| {
        b.iter(|| DualCrc::checksum32(black_box(&bytes_1k)))
    });

    group.bench_function("DualCrc::checksum64 1 kiB", |b| {
        b.iter(|| DualCrc::checksum64(black_box(&bytes_1k)))
    });

    group.bench_function("DualCrc::update 1 kiB", |b| {
        b.iter(|| crc.update(black_box(&bytes_1k)))
    });

    group.bench_function("RollingDualCrc::new 1 kiB", |b| {
        b.iter(|| RollingDualCrc::new(black_box(&bytes_1k)))
    });

    group.throughput(Throughput::Bytes(32 * 1024));

    group.bench_function("RollingDualCrc::new 32 kiB", |b| {
        b.iter(|| RollingDualCrc::new(black_box(&bytes_32k)))
    });

    group.throughput(Throughput::Bytes(1024 * 1024));

    group.bench_function("RollingDualCrc::new 1 MiB", |b| {
        b.iter(|| RollingDualCrc::new(black_box(&bytes_1m)))
    });

    group.throughput(Throughput::Bytes(1));

    group.bench_function("RollingDualCrc::roll 1 kiB", |b| {
        b.iter(|| crc_1k.roll(black_box(b'x')))
    });

    group.bench_function("RollingDualCrc::roll 32 kiB", |b| {
        b.iter(|| crc_32k.roll(black_box(b'x')))
    });

    group.bench_function("RollingDualCrc::roll 1 MiB", |b| {
        b.iter(|| crc_1m.roll(black_box(b'x')))
    });

    group.finish();

    // ============================================================
    // ZEROS

    let mut crc = DualCrc::new();

    let mut group = c.benchmark_group("zeros");
    // use 2^n values here
    for size in [64, 256, 1024] {
        let zeros = Zeros::new(size);
        let zeros_vec = vec![0u8; size];

        group.bench_with_input(
            BenchmarkId::new("update", size),
            &zeros_vec,
            |b, zeros_vec| {
                b.iter(|| {
                    crc.update(zeros_vec);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("update_with_zeros", size),
            &zeros,
            |b, zeros| {
                b.iter(|| {
                    crc.update_with_zeros(&zeros);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Zeros::new(2^n)", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    Zeros::new(size);
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Zeros::new(2^n-1)", size - 1),
            &size,
            |b, &size| {
                b.iter(|| {
                    Zeros::new(size - 1);
                });
            },
        );
    }
    group.finish();
}

criterion_group!(benches, benchmarks);
criterion_main!(benches);
