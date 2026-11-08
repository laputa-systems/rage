use age::{Decryptor, Encryptor, x25519};
use rustybench::{Bencher, counter::BytesCount};
use std::io::{self, Read, Write};
use std::iter;

const KB: usize = 1024;
const SIZES: &[usize] = &[KB, 4 * KB, 16 * KB, 64 * KB, 128 * KB, 256 * KB, 500 * KB, 1024 * KB];

fn main() {
    rustybench::main();
}

#[rustybench::bench(args = SIZES)]
fn encrypt(bencher: Bencher, size: usize) {
    let recipient = x25519::Identity::generate().to_public();
    let plaintext = vec![7u8; size];
    bencher.counter(BytesCount::new(size)).bench_local(|| {
        let mut output = Encryptor::with_recipients(iter::once(&recipient as _))
            .unwrap()
            .wrap_output(io::sink())
            .unwrap();
        output.write_all(rustybench::black_box(&plaintext)).unwrap();
        output.finish().unwrap();
    });
}

#[rustybench::bench(args = SIZES)]
fn decrypt(bencher: Bencher, size: usize) {
    let identity = x25519::Identity::generate();
    let recipient = identity.to_public();
    let plaintext = vec![7u8; size];
    let mut ciphertext = vec![];
    let mut output = Encryptor::with_recipients(iter::once(&recipient as _))
        .unwrap()
        .wrap_output(&mut ciphertext)
        .unwrap();
    output.write_all(&plaintext).unwrap();
    output.finish().unwrap();
    let mut plaintext_out = vec![0u8; size];

    bencher.counter(BytesCount::new(size)).bench_local(|| {
        let decryptor = Decryptor::new_buffered(rustybench::black_box(&ciphertext[..])).unwrap();
        let mut input = decryptor
            .decrypt(iter::once(&identity as &dyn age::Identity))
            .unwrap();
        input.read_exact(&mut plaintext_out).unwrap();
        rustybench::black_box(&plaintext_out);
    });
}
