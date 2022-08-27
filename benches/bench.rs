use criterion::{black_box, criterion_group, criterion_main, Criterion};
use tygress::header::internet::Ipv4;

fn bench(c: &mut Criterion) {
    c.bench_function("bench", |b| {
        b.iter(|| {
            let bytes = [
                0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0, 0xa8,
                0x00, 0x01, 0xc0, 0xa8, 0x00, 0xc7,
            ];
            let header = Ipv4::from_bytes(black_box(&bytes));
            assert!(header.is_ok());
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
