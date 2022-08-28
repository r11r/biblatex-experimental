
use std::collections::HashMap;


mod parse;


#[derive(Debug)]
pub struct Input {
    name: String,
    content: String,
}

impl Input {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        Ok(Input {
            name: path.as_ref().to_string_lossy().into_owned(),
            content: std::fs::read_to_string(path)?,
        })
    }

    fn trace(&self, offset: usize) -> InputTrace {
        // Line number and start of line index
        let mut line = 1;
        let mut start = 0;
        // scan all linebreaks 
        for pos in 0..=offset {
            if let Some(b'\n') = self.content.as_bytes().get(pos) {
                line += 1;
                start = pos+1;
            }
        }
        // count unicode chars in line content
        let col = self.content[start..=offset].chars().count() as u32;
        // full human-friendly trace information
        InputTrace{name: self.name.clone(), line, col}
    }
}

#[derive(Debug)]
pub struct InputTrace {
    pub name: String,
    pub line: u32,
    pub col: u32,
}

#[derive(Debug, Clone)]
struct InputSlice<'de> {
    r#str: &'de str,
    input: &'de Input,
    offset: usize,
}

impl<'de> InputSlice<'de> {

    fn to_string(&self) -> String {
        self.str.to_owned()
    }

    fn trace(&self) -> InputTrace {
        self.input.trace(self.offset)
    }
}


#[derive(Debug)]
pub enum Error{
    InvalidEOF(TokenContext, InputTrace),
    InvalidToken(TokenContext, u8, InputTrace),
    InvalidEntry(String, InputTrace),
    InvalidKey(String, InputTrace),
    InvalidField(String, InputTrace),
    DoubleKey(String, InputTrace, InputTrace),
    DoubleField(String, InputTrace, InputTrace),
    DoubleMacro(String, InputTrace, InputTrace),
    RecursiveMacro(String, InputTrace),
}

#[derive(Debug)]
pub enum TokenContext {
    Global,
    Comment(InputTrace),
    MacroDef(InputTrace),
    Entry(String, InputTrace),
}


#[derive(Debug)]
enum RawValue<'de> {
    Simple(InputSlice<'de>),
    Macro(InputSlice<'de>),
    Compound(Vec<Self>),
}

#[derive(Debug)]
pub struct FieldDef<'de> {
    name: InputSlice<'de>,
    value: RawValue<'de>,
}

#[derive(Debug)]
pub struct RawEntry<'de> {
    entrytype: InputSlice<'de>,
    key: InputSlice<'de>,
    fields: HashMap<&'de str, FieldDef<'de>>
}

type MacroList<'de> = HashMap<&'de str, FieldDef<'de>>;
type RawEntryList<'de> = HashMap<&'de str, RawEntry<'de>>;



pub struct RawBibliography<'de> {
    macros: MacroList<'de>,
    entries: RawEntryList<'de>,
}

impl<'de> RawBibliography<'de> {

    #[inline]
    pub fn new() -> Self {
        Self {
            macros: HashMap::new(),
            entries: HashMap::new(),
        }
    }

    #[inline]
    pub fn add_bibtex_resource(&mut self, input: &'de Input) -> Result<(), Error> {
        Ok(parse::Parser::new(input).parse(self)?)
    }

}



#[cfg(test)]
mod tests {

    fn test_file(file: &str) -> std::path::PathBuf {
        let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("resources/test");
        path.push(file);
        path
    }

    #[test]
    fn biber_benchmark() {

        let inputs = vec![
            super::Input::from_file(test_file("biber-benchmark-definitions.bib")).unwrap(),
            super::Input::from_file(test_file("biber-benchmark-papers.bib")).unwrap(),
        ];

        let mut bib = super::RawBibliography::new();
        for input in &inputs {
            bib.add_bibtex_resource(input).unwrap();
        }

        assert_eq!(bib.macros.len(), 635);
        assert_eq!(bib.entries.len(), 2157);
    }
}

