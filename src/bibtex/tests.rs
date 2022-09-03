
use std::borrow::Cow;

macro_rules! parse {
    ($($lines:expr),+) => {{
        const INPUT: super::Input = super::Input {
            name: Cow::Borrowed("<internal test>"),
            content: Cow::Borrowed(concat!(
                $(
                    $lines,
                    '\n',
                )*
            )),
        };
        super::RawBibliography::new().add_bibtex_resource(&INPUT)
    }};
    // allow for trailing comma
    ($($lines:expr,)+) => {parse!($($lines),+)};
}

// --------------------------

#[test]
fn line_comment() {
    parse!(
        "  % some content to be droped"
    ).unwrap();
}

#[test]
fn comment_block() {
    parse!(
        "@comment {",
        "   some content to be droped",
        "}",
    ).unwrap();
}

#[test]
fn preamble_block() {
    parse!(
        "@preface {",
        "   \\somethingingnored",
        "}"
    ).unwrap();
}

#[test]
fn macro_definition() {
    parse!(
        "@String { ",
        "   macro = {value}, ",
        "}"
    ).unwrap();
}

#[test]
fn minimal_entry() {
    parse!(
        "@entry{key}"
    ).unwrap();
}

#[test]
fn compact_entry() {
    parse!(
        "@type(key,field1={value},field2=\"value\",field3=macro,field4=123)"
    ).unwrap();
}

#[test]
fn sparse_entry() {
    parse!(
        "@type   {   key,  % with comments ",
        "   field1 = { value }  , ",
        "   field2 = \" some other value \"  , %and more comments",
        "   field3 = macro , ",
        "   field4 = 1234  ,  ",
        "   field5 = {",
        "            an complete multiline abstract ",
        "            including \\\"umlauts in brackets % and some comment ",
        "       }  ,",
        "   field6 = \"{\\\"U}nic{\\\"o}de == Ünicöde\"",
        "}",
    ).unwrap();
}

#[test]
fn compound_value_1() {
    parse!(
        "@string ( macro = \"value\")",
        "@entry (key, field = \"extended\" # macro )"
    ).unwrap();
}

#[test]
fn compound_value_2() {
    parse!(
        "@string ( macro = \"value\")",
        "@entry (key, field = macro # \" and more\")"
    ).unwrap();
}
#[test]
fn macro_use() {
    parse!(
        "@string ( macro = \"value\")",
        "@entry (key, field = macro )"
    ).unwrap();
}






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