use biblatex::bibtex;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::path::PathBuf;
//use unicode_normalization::UnicodeNormalization;



fn biber_papers(c: &mut Criterion) {

    // find file
    let testfile = "resources/test/biber-test-papers.bib";
    let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filepath.push(testfile);

    // read file
    let input = black_box(bibtex::Input::from_file(&filepath).unwrap());
    //let content = std::fs::read_to_string(&filepath).unwrap();

    // run
    //c.bench_function("biber papers", |b| b.iter(||{ let _test: String = black_box(&content).chars().nfd().collect();}));
    c.bench_function("biber papers", |b| b.iter(|| bibtex::Parser::new(&input).parse().unwrap()));
}

criterion_group!(benches, biber_papers);
criterion_main!(benches);