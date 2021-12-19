#![feature(bench_black_box)]
use std::hint::black_box;

use tygress::parse::ethernet::EthernetII;

const N_LOOPS: usize = 1_000_000_000;

fn main() {
    let bytes = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 0x08, 0x00];
    let start = std::time::Instant::now();
    for _ in 0..N_LOOPS {
        let header = EthernetII::parse(black_box(&bytes));
        assert!(header.is_ok());
    }

    println!("{:?}", start.elapsed());
}
