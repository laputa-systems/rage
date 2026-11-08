use age::{Decryptor, Encryptor, x25519};
use rustybench::{Bencher, counter::ItemsCount};
use std::io::Write;

fn main() {
    rustybench::main();
}

#[rustybench::bench(args = [1, 2, 3, 4, 5, 6, 7, 8, 9])]
fn parse(bencher: Bencher, count: usize) {
    let recipients: Vec<_> = (0..count)
        .map(|_| x25519::Identity::generate().to_public())
        .collect();
    let mut encrypted = vec![];
    let mut output = Encryptor::with_recipients(recipients.iter().map(|r| r as _))
        .unwrap()
        .wrap_output(&mut encrypted)
        .unwrap();
    output.write_all(&[]).unwrap();
    output.finish().unwrap();

    bencher.counter(ItemsCount::new(count)).bench_local(|| {
        Decryptor::new_buffered(rustybench::black_box(&encrypted[..])).unwrap()
    });
}
