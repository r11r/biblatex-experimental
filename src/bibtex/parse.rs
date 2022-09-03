
use super::*;


// Utility

macro_rules! map_err {
    ( $context:expr, $trace:expr ) => {
        |found| match found {
            Some(other) => Error::InvalidToken($context, other, $trace),
            None => Error::InvalidEOF($context, $trace)
        }
    }    
}

macro_rules! fail {
    ( $context:expr, $found:expr, $trace:expr ) => {
        return Err($found).map_err(map_err!($context, $trace))
    }
}


// Main Parser

#[derive(Debug)]
pub(super) struct Parser<'de>{
    input: &'de Input<'de>,
    index: usize,
    saved_index: usize,
}

impl<'de> Parser<'de>{
    
    pub(super) fn new(input: &'de Input) -> Self {
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

    fn discard_next(&mut self) { 
        self.index += 1;
    }

    fn trace_last(&self) -> InputTrace {
        self.input.trace(self.index-1)
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
                Some(9..=13 | 32) => self.discard_next(),
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
            r#str: &self.input.content[self.saved_index..self.index-1],
            input: self.input,
            offset: self.saved_index,
        }
    }

    // actual token parsing
    // --------------------

    fn parse_identifier(&mut self) -> (InputSlice<'de>, Option<u8>) {
        self.save_index();
        loop {
            match self.next() {
                next @ Some(b'{' | b'}' | b',' | b'#' | b'%' | b'(' |  b')' | b'=') => {
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

    fn parse_braced(&mut self) -> Result<(InputSlice<'de>, Option<u8>), Option<u8>> {
        match self.next() { Some(b'{') => (), other => return Err(other) }
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
                None => return Err(None),
            }
        }
    }

    fn parse_string(&mut self) -> Result<(InputSlice<'de>, Option<u8>), Option<u8>> {
        match self.next() { Some(b'"') => (), other => return Err(other) }
        self.save_index();
        loop {
            match self.next() {
                Some(b'"') => return Ok((self.saved_until_last(), self.next_after_whitespace())),
                Some(b'\\') => self.discard_next(),
                Some(_) => (),
                None => return Err(None),
            }
        }
    }

    fn parse_value(&mut self) -> Result<(RawValue<'de>, Option<u8>), Option<u8>> {
        match self.peek_after_whitespace() {
            Some(b'0'..=b'9') => {
                let (value, next) = self.parse_number();
                Ok((RawValue::Simple(value), next))
            },
            Some(b'{') => {
                let (value, next) = self.parse_braced()?;
                Ok((RawValue::Simple(value), next))
            },
            Some(b'"') => {
                let (value, next) = self.parse_string()?;
                if next == Some(b'#') {
                    todo!("compoound value")
                } else {
                    return Ok((RawValue::Simple(value), next));
                }
            },
            Some(_) => {
                let (name, next) = self.parse_identifier();
                if next == Some(b'#') {
                    todo!("compoound value")
                } else {
                    return Ok((RawValue::Macro(name), next));
                }
            }
            None => return Err(None),
        }
    }

    fn close_brace(&mut self) -> Result<(), Option<u8>>{
        let mut depth = 1;
        loop {
            match self.next() {
                Some(b'{') => depth += 1,
                Some(b'}') => {
                    depth -= 1;
                    if depth == 0 {return Ok(())}
                }
                Some(_) => (),
                None => return Err(None),
            }
        }
    }

    fn close_parenthesis(&mut self) -> Result<(), Option<u8>>{
        loop {
            match self.next() {
                Some(b'{') => self.close_brace()?,
                Some(b')') => return Ok(()),
                Some(_) => (),
                None => return Err(None),
            }
        }
    }

    pub(super) fn parse(&mut self, bib: &mut super::RawBibliography<'de>) -> Result<(), Error> {

        while let Some(byte) = self.next_after_whitespace() {

            if byte != b'@' {
                return Err(Error::InvalidToken(TokenContext::Global, byte, self.trace_last()))
            }

            let (entrytype, closing_braket) = match self.parse_identifier() {
                (entrytype, Some(b'{')) => (entrytype, b'}'),
                (entrytype, Some(b'(')) => (entrytype, b')'),
                (_, other) => fail!(TokenContext::Global, other, self.trace_last()),
            };

            if entrytype.str.eq_ignore_ascii_case("comment") || entrytype.str.eq_ignore_ascii_case("preface") {
                match closing_braket {
                    b'}' => self.close_brace().map_err(map_err!(TokenContext::Comment(entrytype.trace()), self.trace_last()))?,
                    b')' => self.close_parenthesis().map_err(map_err!(TokenContext::Comment(entrytype.trace()), self.trace_last()))?,
                    _ => unreachable!(),
                }
                
            // } else if entrytype.str.eq_ignore_ascii_case("preface") {
            //     todo!("@preface");

            } else if entrytype.str.eq_ignore_ascii_case("string") {
                
                loop {
                    // stop after trailing comma
                    if self.peek_after_whitespace() == Some(closing_braket) {
                        self.discard_next();
                        break;
                    }
                    
                    let name = match self.parse_identifier() {
                        (name, Some(b'=')) => name,
                        (_, other) => fail!(TokenContext::MacroDef(entrytype.trace()), other, self.trace_last()),
                    };

                    if let Some(other) = bib.macros.get(name.str) {
                        return Err(Error::DoubleMacro(name.to_string(), other.name.trace(), name.trace()));
                    } 

                    let (value, next) = self.parse_value().map_err(map_err!(TokenContext::MacroDef(entrytype.trace()), self.trace_last()))?;
                    if let RawValue::Simple(_) = value {
                        bib.macros.insert(name.str, FieldDef{name, value});
                    } else {
                        return Err(Error::RecursiveMacro(name.to_string(), name.trace()));
                    }
                    
                    match next {
                        Some(b',') => continue,
                        Some(byte) if byte == closing_braket => break,
                        other => fail!(TokenContext::MacroDef(entrytype.trace()), other, self.trace_last()),
                    }
                }

            } else {
                self.peek_after_whitespace();

                let (key, mut next) = self.parse_identifier();

                if let Some(other) = bib.entries.get(key.str) {
                    return Err(Error::DoubleKey(key.to_string(), other.key.trace(), key.trace()));
                }
                
                let mut fields: HashMap<&'de str, FieldDef<'de>> = HashMap::new();

                while next == Some(b',') {
                    
                    // stop after trailing comma
                    if self.peek_after_whitespace() == Some(closing_braket) {
                        next = self.next();
                        break;
                    }

                    let name = match self.parse_identifier() {
                        (name, Some(b'=')) => name,
                        (_, other) => fail!(TokenContext::Entry(key.to_string(), key.trace()), other, self.trace_last()),
                    };

                    if let Some(other) = fields.get(name.str) {
                        return Err(Error::DoubleField(name.to_string(), other.name.trace(), name.trace()));
                    } 

                    let value;
                    (value, next) = self.parse_value().map_err(map_err!(TokenContext::MacroDef(entrytype.trace()), self.trace_last()))?;
                    fields.insert(name.str, FieldDef{name, value});
                }

                if next != Some(closing_braket) {
                    fail!(TokenContext::Entry(key.to_string(), key.trace()), next, self.trace_last())
                }

                bib.entries.insert(key.str, RawEntry{entrytype, key, fields});
            }
        }
        Ok(())
    }

}

