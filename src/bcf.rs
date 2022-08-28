use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use quick_xml::de::{from_reader, DeError};


#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Controlfile {
    pub datamodel: Datamodel,
}

impl Controlfile {
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, DeError> {
        let file = std::fs::File::open(path).map_err(|e| quick_xml::Error::from(e))?;
        from_reader(std::io::BufReader::new(file))
    }
}



#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Datamodel {
    constants: ConstantDefinitions,
    entrytypes: EntrytypeDefinitions,
    fields: FieldDefinitions,
    entryfields: Vec<EntrytypesAndFields>,
    multiscriptfields: Fields,
    #[serde(default)] constraints: Vec<ConstraintSet>,
}

impl Datamodel {

    pub fn valid_entrytypes(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        for def in &self.entrytypes.entrytype {
            types.insert(def.name.clone());
        }
        types
    }

    pub fn valid_fields(&self) -> HashSet<String> {
        let mut types = HashSet::new();
        for def in &self.fields.field {
            types.insert(def.name.clone());
        }
        types
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ConstantDefinitions {
    constant: Vec<Constant>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Constant {
    name: String,
    r#type: String,
    #[serde(rename = "$value")]
    value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct EntrytypeDefinitions {
    entrytype: Vec<EntrytypeDefinition>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct EntrytypeDefinition {
    #[serde(rename = "$value")] name: String,
    #[serde(default)] skip_output: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct FieldDefinitions {
    field: Vec<FieldDefinition>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct FieldDefinition {
    #[serde(rename = "$value")] name: String,
    fieldtype: FieldType,
    datatype: DataType,
    #[serde(default)] format: String,
    #[serde(default)] skip_output: bool,
    #[serde(default)] nullok: bool,
    #[serde(default)] label: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum FieldType {
    Field,
    List,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum DataType {
    Name,
    Literal,
    Integer,
    Key,
    Entrykey,
    Date,
    Datepart,
    Verbatim,
    Uri,
    Keyword,
    Option,
    Range,
    Code,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct EntrytypesAndFields {
    #[serde(default)] entrytype: Vec<String>,
    #[serde(default)] field: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Fields {
    #[serde(default)] field: Vec<String>,
}
impl Default for Fields {
    fn default() -> Self { Self{field: vec![]} }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ConstraintSet {
    #[serde(default)] entrytype: Vec<String>,
    constraint: Vec<Constraint>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Constraint {
    r#type: ConstraintType,
    #[serde(default)] datatype: String,
    #[serde(default)] pattern: String,
    #[serde(default)] field: Vec<String>,
    #[serde(default)] fieldor: Fields,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
enum ConstraintType {
    Mandatory,
    Data,
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
    fn deserialize() {
        let _test = super::Controlfile::from_file(test_file("default-datamodel.bcf")).unwrap();
    }

    #[test]
    fn default_datamodel() {
        let controlfile = super::Controlfile::from_file(test_file("default-datamodel.bcf")).unwrap();
        assert_eq!(51, controlfile.datamodel.valid_entrytypes().len());
        assert_eq!(202, controlfile.datamodel.valid_fields().len());
    }
}