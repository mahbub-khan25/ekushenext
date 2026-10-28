//! Write the word pairs stored in a packed dictionary as `ekushe-dict pack --pairs` input.
//! `dump_pairs DICT.z OUT.tsv`
use ekushe_core::dict::Dictionary;
use std::io::Write;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    let d = Dictionary::from_compressed(&std::fs::read(&a[1]).unwrap()).unwrap();
    let mut w = std::io::BufWriter::new(std::fs::File::create(&a[2]).unwrap());
    let mut n = 0;
    for id in 0..d.len() as u32 {
        for (next, c) in d.followers(id) {
            writeln!(w, "{}\t{}\t{}", d.word(id), d.word(next), c).unwrap();
            n += 1;
        }
    }
    eprintln!("{n} pairs");
}
