

pub mod bibtex;




#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    #[test]
    fn it_works() {

        // find file
        let testfile = "resources/test/biber-test-papers.bib";
        let mut filepath = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        filepath.push(testfile);


        // read file
        let content = super::bibtex::Input::from_file(filepath).unwrap();


        let map = super::bibtex::Parser::new(&content).parse().unwrap();

        // for (k, v) in map.iter().take(15) {
        //     println!("{} '{}' [{} Fields]", v.entrytype, k, v.fields.len());
        // }
        // println!("...");

        // for (k, v) in map.iter().take(1) {
        //     println!("{:?}", v);
        // }

        assert_eq!(map.len(), 2157);
        
        //println!("{}", content);
    }
}

