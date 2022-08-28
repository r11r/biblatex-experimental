use biblatex::{bibtex};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
//use unicode_normalization::UnicodeNormalization;


fn test_file(file: &str) -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("resources/test");
    path.push(file);
    black_box(path)
}


fn biber_papers(c: &mut Criterion) {

    // --- IO ---
    let inputs = vec![
        bibtex::Input::from_file(test_file("biber-benchmark-definitions.bib")).unwrap(),
        bibtex::Input::from_file(test_file("biber-benchmark-papers.bib")).unwrap(),
    ];

    c.bench_function("biber papers", |b| b.iter(|| {

        // --- Parsing ---
        let mut bib = bibtex::RawBibliography::new();
        for input in &inputs {
            bib.add_bibtex_resource(input).unwrap();
        }

    }));
    
}

criterion_group!(benches, biber_papers);
criterion_main!(benches);