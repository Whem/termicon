//! Throughput benchmarks

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

fn codec_benchmark(c: &mut Criterion) {
    let data: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();

    let mut group = c.benchmark_group("codec");
    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("hex_encode", |b| {
        b.iter(|| {
            let encoded = hex::encode(black_box(&data));
            black_box(encoded)
        })
    });

    group.bench_function("hex_decode", |b| {
        let hex_str = hex::encode(&data);
        b.iter(|| {
            let decoded = hex::decode(black_box(&hex_str)).unwrap();
            black_box(decoded)
        })
    });

    group.finish();
}

fn trigger_benchmark(c: &mut Criterion) {
    let data: Vec<u8> = b"This is a test message with ERROR: 123 somewhere in it and more text after".to_vec();

    let mut group = c.benchmark_group("trigger");

    group.bench_function("text_match", |b| {
        b.iter(|| {
            let found = black_box(&data)
                .windows(5)
                .any(|w| w == b"ERROR");
            black_box(found)
        })
    });

    group.bench_function("regex_match", |b| {
        let re = regex::Regex::new(r"ERROR:\s+\d+").unwrap();
        let text = String::from_utf8_lossy(&data);
        b.iter(|| {
            let found = re.is_match(black_box(&text));
            black_box(found)
        })
    });

    group.finish();
}

criterion_group!(benches, codec_benchmark, trigger_benchmark);
criterion_main!(benches);








