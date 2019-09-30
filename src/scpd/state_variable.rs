use crate::find_in_xml;
use crate::Error;
use roxmltree::Node;
use std::fmt;

#[derive(Debug)]
pub struct StateVariable {
    name: String,
    datatype: DataType,
    default: Option<String>,
    allowed_values: Option<Vec<String>>,
    allowed_range: Option<AllowedValueRange>,
    optional: bool,
}
impl fmt::Display for StateVariable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl StateVariable {
    pub(crate) fn from_xml(node: Node) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (name, datatype, default, allowed_values, allowed_range, optional) = find_in_xml! { node => name, dataType, ?defaultValue, ?allowedValueList, ?allowedValueRange, ?optional };

        let allowed_range = allowed_range.map(AllowedValueRange::from_xml).transpose()?;
        let allowed_values = allowed_values
            .map(|a: Node| {
                a.children()
                    .filter(Node::is_element)
                    .map(crate::parse_node_text)
                    .collect()
            })
            .transpose()?;
        let default = default.map(crate::parse_node_text).transpose()?;

        Ok(Self {
            name: crate::parse_node_text(name)?,
            datatype: crate::parse_node_text(datatype)?,
            default,
            allowed_values,
            allowed_range,
            optional: optional.is_some(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name //.trim_start_matches("A_ARG_TYPE_")
    }

    pub fn datatype(&self) -> DataType {
        self.datatype
    }

    pub fn allowed_values(&self) -> Option<&Vec<String>> {
        self.allowed_values.as_ref()
    }

    pub fn allowed_value_range(&self) -> Option<&AllowedValueRange> {
        self.allowed_range.as_ref()
    }

    pub fn default_value(&self) -> Option<&String> {
        self.default.as_ref()
    }
}

#[derive(Debug, Copy, Clone)]
#[allow(non_camel_case_types)]
pub enum DataType {
    ui1,
    ui2,
    ui4,
    ui8,
    i1,
    i2,
    i4,
    int,
    r4,
    r8,
    Number,
    Float,
    Fixed14_4,
    Char,
    String,
    Date,
    DateTime,
    DateTimeTz,
    Time,
    TimeTz,
    Boolean,
    BinBase64,
    BinHex,
    Uri,
}
impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug)]
pub struct ParseDataTypeErr(String);
impl fmt::Display for ParseDataTypeErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid data type: {}", self.0)
    }
}
impl std::error::Error for ParseDataTypeErr {}
impl std::str::FromStr for DataType {
    type Err = ParseDataTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ui1" => Ok(DataType::ui1),
            "ui2" => Ok(DataType::ui2),
            "ui4" => Ok(DataType::ui4),
            "ui8" => Ok(DataType::ui8),
            "i1" => Ok(DataType::i1),
            "i2" => Ok(DataType::i2),
            "i4" => Ok(DataType::i4),
            "int" => Ok(DataType::int),
            "r4" => Ok(DataType::r4),
            "r8" => Ok(DataType::r8),
            "number" => Ok(DataType::Number),
            "float" => Ok(DataType::Float),
            "fixed14_4" => Ok(DataType::Fixed14_4),
            "char" => Ok(DataType::Char),
            "string" => Ok(DataType::String),
            "date" => Ok(DataType::Date),
            "dateTime" => Ok(DataType::DateTime),
            "dateTimeTz" => Ok(DataType::DateTimeTz),
            "time" => Ok(DataType::Time),
            "timeTz" => Ok(DataType::TimeTz),
            "boolean" => Ok(DataType::Boolean),
            "binBase64" => Ok(DataType::BinBase64),
            "binHex" => Ok(DataType::BinHex),
            "uri" => Ok(DataType::Uri),
            _ => Err(ParseDataTypeErr(s.to_string())),
        }
    }
}

#[derive(Debug)]
pub struct AllowedValueRange {
    ///Inclusive lower bound
    minimum: i64,
    ///Inclusive upper bound.
    maximum: i64,
    step: i64,
}
impl AllowedValueRange {
    fn from_xml(node: Node) -> Result<Self, Error> {
        #[allow(non_snake_case)]
        let (minimum, maximum, step) = find_in_xml! { node => minimum, maximum, ?step };

        let step = step.map(crate::parse_node_text).transpose()?.unwrap_or(1);

        Ok(Self {
            minimum: crate::parse_node_text(minimum)?,
            maximum: crate::parse_node_text(maximum)?,
            step,
        })
    }
    pub fn minimum(&self) -> i64 {
        self.minimum
    }
    pub fn maximum(&self) -> i64 {
        self.maximum
    }
    pub fn step(&self) -> i64 {
        self.step
    }
}
