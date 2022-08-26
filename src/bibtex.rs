use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::ops::Range;
use std::collections::HashMap;
use std::path::Path;


#[derive(Debug)]
pub enum Error{
    UnexpectedChar(u8, u8),
    UnexpectedEOF(u8),
    DoubleField,
    DoubleKey,
}


#[derive(Debug)]
pub struct Input {
    name: String,
    content: String,
}

impl Input {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        Ok(Input {
            name: path.as_ref().to_string_lossy().into_owned(),
            content: std::fs::read_to_string(path)?,
        })
    }
}


#[derive(Debug, Clone)]
pub struct InputSlice<'de> {
    input: &'de Input,
    range: Range<usize>,
}

impl<'de> InputSlice<'de> {
    pub fn trace(&self) -> (String, usize, usize) {
        // Line number and start of line index
        let mut line = 1;
        let mut start = 0;
        // scan all linebreaks
        for pos in 0..=self.range.start {
            if let Some(b'\n') = &self.input.content.as_bytes().get(pos) {
                line += 1;
                start = pos+1;
            }
        }
        // count unicode chars in line content
        let col = self.input.content[start..=self.range.start].chars().count();
        // full human-friendly trace information
        (self.input.name.clone(), line, col)
    }
}

impl<'de> Deref for InputSlice<'de> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        // PANIC: This can panic if char-borders are invalid!
        &self.input.content[self.range.clone()]
    }
}


impl<'de> Hash for InputSlice<'de> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.deref().hash(state);
    }
}

impl<'de> PartialEq for InputSlice<'de> {
    fn eq(&self, other: &Self) -> bool {
        self.deref() == other.deref()
    }
}

impl<'de> Eq for InputSlice<'de> {}



#[derive(Debug)]
enum Value<'de> {
    Simple(InputSlice<'de>),
    Macro(InputSlice<'de>),
    Compound(Vec<Self>),
}

#[derive(Debug)]
pub struct Entry<'de> {
    entrytype: InputSlice<'de>,
    key: InputSlice<'de>,
    fields: HashMap<InputSlice<'de>, Value<'de>>
}



#[derive(Debug)]
pub struct Parser<'de>{
    input: &'de Input,
    index: usize,
    saved_index: usize,
}

impl<'de> Parser<'de>{
    
    pub fn new(input: &'de Input) -> Self {
        Parser {
            input: input,
            index: 0,
            saved_index: 0,
        }
    }
   

    // Base Functions
    // --------------

    fn peek(&self) -> Option<u8> {
        // fastest implementation arcording to serde_json::read::SliceRead
        if self.index < self.input.content.len() {
            let ch = self.input.content.as_bytes()[self.index];
            Some(ch)
        } else {
            None
        }
    }

    fn next(&mut self) -> Option<u8> { 
        // fastest implementation arcording to serde_json::read::SliceRead
        if self.index < self.input.content.len() {
            let ch = self.input.content.as_bytes()[self.index];
            self.index += 1;
            Some(ch)
        } else {
            None
        }
    }

    fn discard(&mut self) { 
        self.index += 1;
    }


    // Whitespace and comments
    // -----------------------

    fn discard_line(&mut self) {
        loop {
            match self.next() {
                Some(b'\n') => return,
                _ => (),
            }
        }
    }

    fn peek_after_whitespace(&mut self) -> Option<u8> {
        loop {
            match self.peek() {
                Some(9..=13 | 32) => self.discard(),
                Some(b'%') => self.discard_line(),
                result => return result,
            }
        }
    }

    fn next_after_whitespace(&mut self) -> Option<u8> {
        loop {
            match self.next() {
                Some(9..=13 | 32) => (),
                Some(b'%') => self.discard_line(),
                result => return result,
            }
        }
    }


    // String recording
    // ----------------

    fn save_index(&mut self) {
        self.saved_index = self.index;
    }

    fn saved_until_last(&self) -> InputSlice<'de> {
        InputSlice {
            input: self.input,
            range: self.saved_index..self.index-1,
        }
        // PANIC: correct byte borders have to be enforced by parser functions!
        //&self.input.content[self.saved_index..self.index-1]
        // // this would be failsafe
        // String::from_utf8_lossy(&self.input.content.as_bytes()[self.saved_index..self.index-1])
    }


    // actual token parsing
    // --------------------

    fn assert_next(&mut self, expected: u8) -> Result<(), Error> {
        match self.next() {
            Some(byte) if byte == expected => Ok(()),
            Some(other) => Err(Error::UnexpectedChar(other, expected)),
            None => Err(Error::UnexpectedEOF(expected)),
        }
    }

    fn parse_entrytype(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                next @ Some(b'{' | b'(') => {
                    return (self.saved_until_last(), next)
                },
                Some(9..=13 | 32) | None => {
                    return (self.saved_until_last(), self.next_after_whitespace())
                },
                _ => (),
            }
        }
    }

    fn parse_entrykey(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                next @ Some(b',' | b'}' | b')') => {
                    return (self.saved_until_last(), next)
                },
                Some(9..=13 | 32) | None => {
                    return (self.saved_until_last(), self.next_after_whitespace())
                },
                _ => (),
            }
        }
    }

    fn parse_fieldname(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                next @ Some(b'=') => {
                    return (self.saved_until_last(), next)
                },
                Some(9..=13 | 32) | None => {
                    return (self.saved_until_last(), self.next_after_whitespace())
                },
                _ => (),
            }
        }
    }

    fn parse_number(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                Some(b'0'..=b'9') => (),
                Some(9..=13 | 32) => {
                    return (self.saved_until_last(), self.next_after_whitespace())
                },
                next => return (self.saved_until_last(), next),
            }
        }
    }

    fn parse_braced(&mut self) -> Result<(InputSlice<'de>, Option<u8>), Error> {
        self.assert_next(b'{')?;
        self.save_index();
        let mut depth = 1;
        loop {
            match self.next() {
                Some(b'{') => depth += 1,
                Some(b'}') => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok((self.saved_until_last(), self.next_after_whitespace()))
                    }
                }
                Some(_) => (),
                None => return Err(Error::UnexpectedEOF(b'}')),
            }
        }
    }

    fn parse_string(&mut self) -> Result<(InputSlice<'de>, Option<u8>), Error> {
        self.assert_next(b'"')?;
        self.save_index();
        loop {
            match self.next() {
                Some(b'"') => return Ok((self.saved_until_last(), self.next_after_whitespace())),
                Some(_) => (),
                None => return Err(Error::UnexpectedEOF(b'"')),
            }
        }
    }

    fn parse_macro(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                next @ Some(b'#' | b',' | b')' | b'}') => return (self.saved_until_last(), next),
                Some(9..=13 | 32) => {
                    return (self.saved_until_last(), self.next_after_whitespace())
                },
                Some(_) => (),
                None => return (self.saved_until_last(), None),
            }
        }
    }

    fn parse_value(&mut self) -> Result<(Value<'de>, Option<u8>), Error> {
        match self.peek_after_whitespace() {
            Some(b'0'..=b'9') => {
                let (value, next) = self.parse_number();
                Ok((Value::Simple(value), next))
            },
            Some(b'{') => {
                let (value, next) = self.parse_braced()?;
                Ok((Value::Simple(value), next))
            },
            Some(b'"') => {
                let (value, next) = self.parse_string()?;
                if next == Some(b'#') {
                    todo!("compoound value")
                } else {
                    return Ok((Value::Simple(value), next));
                }
            },
            Some(_) => {
                let (name, next) = self.parse_macro();
                if next == Some(b'#') {
                    todo!("compoound value")
                } else {
                    return Ok((Value::Macro(name), next));
                }
            }
            None => return Err(Error::UnexpectedEOF(b'"')),
        }
    }

    fn close_brace(&mut self) -> Result<(), Error>{
        let mut depth = 1;
        loop {
            match self.next() {
                Some(b'{') => depth += 1,
                Some(b'}') => {
                    depth -= 1;
                    if depth == 0 {return Ok(())}
                }
                Some(_) => (),
                None => return Err(Error::UnexpectedEOF(b'}')),
            }
        }
    }

    fn close_parenthesis(&mut self) -> Result<(), Error>{
        loop {
            match self.next() {
                Some(b'{') => self.close_brace()?,
                Some(b')') => return Ok(()),
                Some(_) => (),
                None => return Err(Error::UnexpectedEOF(b')')),
            }
        }
    }

    pub fn parse(&mut self) -> Result<HashMap<InputSlice<'de>, Entry<'de>>, Error> {
        let mut entries = HashMap::new();
        let mut next: Option<u8>;
        loop {
            match self.next_after_whitespace() {
                Some(b'@') => {

                    let entrytype;
                    (entrytype, next) = self.parse_entrytype();

                    let closing: u8;
                    match next {
                        Some(b'{') => closing = b'}',
                        Some(b'(') => closing = b')',
                        Some(other) => return Err(Error::UnexpectedChar(other, b'{')),
                        None => return Err(Error::UnexpectedEOF(b'{')),
                    }

                    if entrytype.eq_ignore_ascii_case("comment") {
                        match closing {
                            b'}' => self.close_brace()?,
                            b')' => self.close_parenthesis()?,
                            _ => unreachable!(),
                        }
                        
                    } else if entrytype.eq_ignore_ascii_case("preface") {
                        todo!("@preface");

                    } else if entrytype.eq_ignore_ascii_case("string") {
                        todo!("@string");

                    } else {
                        let key;
                        (key, next) = self.parse_entrykey();
                        let mut fields = HashMap::new();

                        // this supports trailing commas in the list of fields
                        while next == Some(b',') && self.peek_after_whitespace() != Some(closing) {
                            let name;
                            (name, next) = self.parse_fieldname();

                            match next {
                                Some(b'=') => (),
                                Some(other) => return Err(Error::UnexpectedChar(other, b'=')),
                                None => return Err(Error::UnexpectedEOF(b'=')),
                            }

                            let value;
                            (value, next) = self.parse_value()?;

                            if fields.contains_key(&name) {
                                return Err(Error::DoubleField);
                            } else {
                                fields.insert(name, value);
                            }

                        }

                        if next != Some(closing) {
                            if next == Some(b',') {
                                // case of traling comma in entry,
                                // this should be true if the loop has stopped
                                // important: next() or discard() has to be called here!
                                self.assert_next(closing)?
                            } else {
                                match next {
                                    Some(other) => return Err(Error::UnexpectedChar(other, closing)),
                                    None => return Err(Error::UnexpectedEOF(closing)),
                                }
                            }
                        }

                        if entries.contains_key(&key) {
                            return Err(Error::DoubleKey);
                        } else {
                            entries.insert(key.clone(), Entry{entrytype, key, fields});
                        }
                    }
                },
                Some(b) => return Err(Error::UnexpectedChar(b, b'@')),
                None => break,
            }
        }
        Ok(entries)
    }

}




