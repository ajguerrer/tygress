#![feature(bench_black_box)]
use std::hint::black_box;

use tygress::header::internet::Ipv4;

const N_LOOPS: usize = 1_000_000_000;

fn main() {
    let bytes = [
        0x45, 0x00, 0x00, 0x73, 0x00, 0x00, 0x40, 0x00, 0x40, 0x11, 0xb8, 0x61, 0xc0, 0xa8, 0x00,
        0x01, 0xc0, 0xa8, 0x00, 0xc7,
    ];
    let start = std::time::Instant::now();
    for _ in 0..N_LOOPS {
        let header = Ipv4::from_bytes(black_box(&bytes));
        assert!(header.is_ok());
    }

    println!("{:?}", start.elapsed());
}
